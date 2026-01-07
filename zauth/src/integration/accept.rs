use crate::{codec::AuthPayloadCodec, session::AuthSession, AuthMessage, AuthStore, Authenticator};
use futures::StreamExt;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio_util::codec::{Encoder, FramedRead};
use tracing::error;
use zwire::{
    codec::FrameCodec,
    errors::WireError,
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
) -> Result<bool, WireError> {
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
            error!("Received frame isn't an auth payload");

            return Ok(false);
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

#[allow(clippy::too_many_arguments)]
async fn ensure_auth<S: AsyncWrite + std::marker::Unpin, R: AsyncRead + std::marker::Unpin>(
    session_manager: Arc<SessionManager<impl SessionBackend>>,
    authenticator: Arc<Authenticator<impl AuthStore>>,
    connection_id: usize,
    send: &mut S,
    receive: R,
    codec_buffer: &mut BytesMut,
    mut frame_codec: FrameCodec,
    auth_payload_codec: &mut AuthPayloadCodec,
) -> Result<bool, WireError> {
    let mut auth_status = perform_auth(
        None,
        send,
        connection_id,
        session_manager.clone(),
        authenticator.clone(),
        codec_buffer,
        &mut frame_codec,
        auth_payload_codec,
    )
    .await?;

    if auth_status {
        return Ok(auth_status);
    };

    let mut framed_reader = FramedRead::new(receive, frame_codec);

    if let Some(Ok(frame)) = framed_reader.next().await {
        auth_status = perform_auth(
            Some(frame),
            send,
            connection_id,
            session_manager.clone(),
            authenticator.clone(),
            codec_buffer,
            &mut frame_codec,
            auth_payload_codec,
        )
        .await?;
    }

    Ok(auth_status)
}

#[cfg(feature = "quinn_integration")]
pub trait AcceptAuthed {
    fn accept_authed(
        &self,
        session_manager: Arc<SessionManager<impl SessionBackend>>,
        authenticator: Arc<Authenticator<impl AuthStore>>,
    ) -> impl std::future::Future<Output = Result<Option<quinn::Connection>, quinn::ConnectionError>>
           + Send;
}

#[cfg(feature = "quinn_integration")]
impl AcceptAuthed for quinn::Endpoint {
    async fn accept_authed(
        &self,
        session_manager: Arc<SessionManager<impl SessionBackend>>,
        authenticator: Arc<Authenticator<impl AuthStore>>,
    ) -> Result<Option<quinn::Connection>, quinn::ConnectionError> {
        let Some(incoming) = self.accept().await else {
            return Ok(None);
        };

        match incoming.accept() {
            Err(error) => Err(error),
            Ok(connecting) => match connecting.await {
                Err(error) => Err(error),
                Ok(connection) => {
                    let (mut send, receive) = connection.open_bi().await?;
                    let mut codec_buffer = BytesMut::new();

                    match ensure_auth(
                        session_manager,
                        authenticator,
                        connection.stable_id(),
                        &mut send,
                        receive,
                        &mut codec_buffer,
                        FrameCodec::default(),
                        &mut AuthPayloadCodec::default(),
                    )
                    .await
                    {
                        Err(_error) => todo!(), // TODO: Error handling
                        Ok(auth_status) => {
                            if auth_status {
                                Ok(Some(connection))
                            } else {
                                Ok(None)
                            }
                        }
                    }
                }
            },
        }
    }
}
