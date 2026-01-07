use crate::{codec::AuthPayloadCodec, AuthMessage, AuthPayload};
use futures::TryStreamExt;
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::time::{sleep, timeout, Duration};
use tokio_util::codec::{Encoder, FramedRead};
use tracing::{error, warn};
use zwire::{
    codec::bytes::ByteStr, codec::FrameCodec, errors::WireError, BytesMut, EncodeIntoFrame, Frame,
};

async fn send_auth_frame<S: AsyncWrite + std::marker::Unpin>(
    codec_buffer: &mut BytesMut,
    send: &mut S,
    client_identifier: ByteStr,
    key: &str,
    frame_codec: &mut FrameCodec,
    auth_payload_codec: &mut AuthPayloadCodec,
) {
    let auth_payload = AuthPayload::new(client_identifier, key).unwrap();
    let frame = auth_payload_codec
        .encode_into_frame(auth_payload, AuthMessage::Auth, codec_buffer)
        .unwrap();

    frame_codec.encode(frame, codec_buffer).unwrap();
    send.write_all_buf(codec_buffer).await.unwrap();
    codec_buffer.clear();
}

async fn await_auth_response_frame(
    framed_reader: &mut FramedRead<impl AsyncRead + Unpin, FrameCodec>,
    timeout_duration: Duration,
) -> Result<Option<(AuthMessage, Frame)>, WireError> {
    let result = timeout(timeout_duration, async {
        while let Some(frame) = framed_reader.try_next().await? {
            match AuthMessage::try_from(&frame.message) {
                Err(_) => continue,
                Ok(auth_message) => {
                    return Ok(Some((auth_message, frame)));
                }
            };
        }

        Ok::<Option<(AuthMessage, Frame)>, WireError>(None)
    })
    .await;

    match result {
        Ok(Ok(Some(response))) => Ok(Some(response)),
        Ok(Ok(None)) => {
            error!("EOF before auth response was received");

            Ok(None)
        }
        Ok(Err(error)) => Err(error),
        Err(_elapsed) => {
            error!("Hit timeout while waiting for auth response");

            Ok(None)
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn ensure_auth<S: AsyncWrite + std::marker::Unpin, R: AsyncRead + std::marker::Unpin>(
    client_identifier: ByteStr,
    key: &str,
    max_retries: u64,
    retry_cooldown: Duration,
    mut frame_codec: FrameCodec,
    auth_payload_codec: &mut AuthPayloadCodec,
    send: &mut S,
    receive: &mut R,
) -> Result<bool, quinn::ConnectionError> {
    let mut codec_buffer = BytesMut::new();
    let mut framed_reader = FramedRead::new(receive, frame_codec);
    let mut retries = 0;

    const FRAME_RECEIVE_TIMEOUT: u64 = 1;

    match await_auth_response_frame(
        &mut framed_reader,
        Duration::from_secs(FRAME_RECEIVE_TIMEOUT),
    )
    .await
    {
        Err(error) => error!("{:#?} | on await_auth_response_frame", error),
        Ok(response) => {
            if let Some((auth_message, _frame)) = response {
                match auth_message {
                    AuthMessage::Auth => (),
                    AuthMessage::AuthRequired => (),
                    AuthMessage::AuthInvalid => (),
                    AuthMessage::AuthValid => return Ok(true),
                }
            } else {
                // No auth response was sent by server, not authenticated
                error!("No auth response was sent by server, not authenticated. Maybe endpoint doesn't require Auth? Or something is wrong with the Server.");
                return Ok(false);
            }
        }
    }

    // Don't count first try as a retry
    while retries < (max_retries + 1) {
        send_auth_frame(
            &mut codec_buffer,
            send,
            client_identifier.clone(),
            key,
            &mut frame_codec,
            auth_payload_codec,
        )
        .await;

        let auth_status = match await_auth_response_frame(
            &mut framed_reader,
            Duration::from_secs(FRAME_RECEIVE_TIMEOUT),
        )
        .await
        {
            Err(error) => {
                error!("{:#?} | on await_auth_response_frame", error);

                return Ok(false);
            }
            Ok(response) => {
                if let Some((auth_message, _frame)) = response {
                    match auth_message {
                        AuthMessage::Auth => false,
                        AuthMessage::AuthRequired => false,
                        AuthMessage::AuthInvalid => false,
                        AuthMessage::AuthValid => true,
                    }
                } else {
                    // No auth response was sent by server, again
                    error!("Server isn't sending auth response");

                    return Ok(false);
                }
            }
        };

        if auth_status {
            return Ok(true);
        } else {
            retries += 1;

            warn!("Client failed to authorize, retrying");

            sleep(retry_cooldown).await;
        }
    }

    Ok(false)
}

#[cfg(feature = "quinn_integration")]
pub trait ConnectAuthed {
    fn connect_with_authed(
        &self,
        config: quinn::ClientConfig,
        address: SocketAddr,
        server_name: &str,
        client_identifier: ByteStr,
        key: &str,
    ) -> impl std::future::Future<Output = Result<Option<quinn::Connection>, quinn::ConnectionError>>
           + Send;
}

#[cfg(feature = "quinn_integration")]
impl ConnectAuthed for quinn::Endpoint {
    async fn connect_with_authed(
        &self,
        config: quinn::ClientConfig,
        address: SocketAddr,
        server_name: &str,
        client_identifier: ByteStr,
        key: &str,
    ) -> Result<Option<quinn::Connection>, quinn::ConnectionError> {
        const RETRY_COOLDOWN: u64 = 1;
        const MAX_RETRIES: u64 = 0;

        match self.connect_with(config, address, server_name) {
            Err(_error) => todo!(), // Well this is a connect error, we needa break up the function
            // into two like quinn does first returns connect error next connection Error but that
            // can wait for a bit
            Ok(connecting) => match connecting.await {
                Err(error) => Err(error),
                Ok(connection) => {
                    let (mut send, mut receive) = connection.accept_bi().await?;

                    let auth_status = ensure_auth(
                        client_identifier,
                        key,
                        MAX_RETRIES,
                        Duration::from_secs(RETRY_COOLDOWN),
                        FrameCodec::default(),
                        &mut AuthPayloadCodec::default(),
                        &mut send,
                        &mut receive,
                    )
                    .await?;

                    let _ = receive.stop(quinn::VarInt::from_u32(0));
                    let _ = send.finish();

                    if auth_status {
                        Ok(Some(connection))
                    } else {
                        Ok(None)
                    }
                }
            },
        }
    }
}
