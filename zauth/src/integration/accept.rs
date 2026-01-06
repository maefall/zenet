use crate::{codec::AuthPayloadCodec, session::AuthSession, AuthMessage, AuthStore, Authenticator};
use std::sync::Arc;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio_util::codec::{Encoder, FramedRead};
use tracing::info;
use zwire::{
    codec::FrameCodec,
    session::{SessionBackend, SessionManager},
    BytesMut, DecodeFromFrame, Frame,
};

#[allow(clippy::too_many_arguments)]
async fn perform_auth<S: AsyncWrite + std::marker::Unpin>(
    frame: Option<Frame>,
    send: &mut S,
    connection_id: usize,
    session_manager: Arc<SessionManager<impl SessionBackend>>,
    authenticator: Arc<Authenticator<impl AuthStore>>,
    codec_buffer: &mut BytesMut,
    frame_codec: &mut FrameCodec,
    auth_payload_codec: &mut AuthPayloadCodec,
) -> Result<bool, anyhow::Error> {
    let mut auth_status_response = false;

    if let Some(frame) = frame {
        if let Some(auth_payload) = auth_payload_codec.decode_from_frame(frame, codec_buffer)? {
            let (auth_status, auth_response_frame) =
                authenticator.process_auth_payload(&auth_payload.0);

            if auth_status {
                session_manager
                    .authenticate(connection_id, auth_payload.0.client_identifier.to_string());

                auth_status_response = true;
            }

            frame_codec.encode(auth_response_frame, codec_buffer)?;
            send.write_all_buf(codec_buffer).await?;
            codec_buffer.clear();
        } else {
            panic!("unexpected frame, DELETE THSI TDOO")
        }
    } else {
        let payload = if session_manager.is_authenticated(connection_id) {
            auth_status_response = true;

            Frame::message_only(AuthMessage::AuthValid)
        } else {
            auth_status_response = false;

            Frame::message_only(AuthMessage::AuthRequired)
        };

        frame_codec.encode(payload, codec_buffer)?;
        send.write_all_buf(codec_buffer).await?;
        codec_buffer.clear();
    }

    Ok(auth_status_response)
}

#[cfg(feature = "quinn")]
async fn ensure_auth(
    session_manager: Arc<SessionManager<impl SessionBackend>>,
    authenticator: Arc<Authenticator<impl AuthStore>>,
    connection: &quinn::Connection,
    mut frame_codec: FrameCodec,
    auth_payload_codec: &mut AuthPayloadCodec,
) -> Result<bool, anyhow::Error> {
    use futures::StreamExt;

    let (mut send, recv) = connection.open_bi().await?;
    let mut codec_buffer = BytesMut::new();

    let mut auth_status = perform_auth(
        None,
        &mut send,
        connection.stable_id(),
        session_manager.clone(),
        authenticator.clone(),
        &mut codec_buffer,
        &mut frame_codec,
        auth_payload_codec,
    )
    .await?;

    if auth_status {
        return Ok(auth_status);
    };

    let mut framed_reader = FramedRead::new(recv, frame_codec);

    if let Some(Ok(frame)) = framed_reader.next().await {
        auth_status = perform_auth(
            Some(frame),
            &mut send,
            connection.stable_id(),
            session_manager.clone(),
            authenticator.clone(),
            &mut codec_buffer,
            &mut frame_codec,
            auth_payload_codec,
        )
        .await?;
    }

    Ok(auth_status)
}

#[cfg(feature = "quinn")]
async fn accept_authed_connection(
    endpoint: &quinn::Endpoint,
    session_manager: Arc<SessionManager<impl SessionBackend>>,
    authenticator: Arc<Authenticator<impl AuthStore>>,
    frame_codec: FrameCodec,
    auth_payload_codec: &mut AuthPayloadCodec,
) -> Option<Result<quinn::Connection, quinn::ConnectionError>> {
    let incoming = endpoint.accept().await?;

    match incoming.accept() {
        Err(error) => Some(Err(error)),
        Ok(connecting) => match connecting.await {
            Err(error) => Some(Err(error)),
            Ok(connection) => {
                info!("Client [{}] connected", connection.remote_address());

                match ensure_auth(
                    session_manager,
                    authenticator,
                    &connection,
                    frame_codec,
                    auth_payload_codec,
                )
                .await
                {
                    Err(_error) => None, // TODO: Error handling
                    Ok(auth_status) => {
                        if auth_status {
                            info!("Client passed authentication");

                            Some(Ok(connection))
                        } else {
                            info!("Client failed authentication");

                            None
                        }
                    }
                }
            }
        },
    }
}

#[cfg(feature = "quinn")]
pub trait AcceptAuthed {
    fn accept_authed(
        &self,
        session_manager: Arc<SessionManager<impl SessionBackend>>,
        authenticator: Arc<Authenticator<impl AuthStore>>,
    ) -> impl std::future::Future<Output = Option<Result<quinn::Connection, quinn::ConnectionError>>>
           + Send;
}

#[cfg(feature = "quinn")]
impl AcceptAuthed for quinn::Endpoint {
    async fn accept_authed(
        &self,
        session_manager: Arc<SessionManager<impl SessionBackend>>,
        authenticator: Arc<Authenticator<impl AuthStore>>,
    ) -> Option<Result<quinn::Connection, quinn::ConnectionError>> {
        accept_authed_connection(
            self,
            session_manager,
            authenticator,
            FrameCodec::default(),
            &mut AuthPayloadCodec::default(),
        )
        .await
    }
}
