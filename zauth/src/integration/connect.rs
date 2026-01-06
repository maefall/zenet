use crate::{codec::AuthPayloadCodec, AuthMessage, AuthPayload};
use std::net::SocketAddr;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::time::{sleep, Duration};
use tokio_util::codec::{Encoder, FramedRead};
use tracing::info;
use zwire::{codec::bytes::ByteStr, codec::FrameCodec, BytesMut, EncodeIntoFrame, Message};

async fn send_auth<S: AsyncWrite + std::marker::Unpin>(
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

async fn perform_auth<S: AsyncWrite + std::marker::Unpin>(
    codec_buffer: &mut BytesMut,
    frame_message: &Message,
    send: &mut S,
    client_identifier: ByteStr,
    key: &str,
    frame_codec: &mut FrameCodec,
    auth_payload_codec: &mut AuthPayloadCodec,
) -> bool {
    if let Ok(auth_message) = AuthMessage::try_from(frame_message) {
        match auth_message {
            AuthMessage::AuthRequired => {
                info!("Server requires auth, sending auth payload");

                send_auth(
                    codec_buffer,
                    send,
                    client_identifier,
                    key,
                    frame_codec,
                    auth_payload_codec,
                )
                .await;

                false
            }
            AuthMessage::AuthInvalid => {
                info!("Server says auth is NOT valid, sending auth payload");

                send_auth(
                    codec_buffer,
                    send,
                    client_identifier,
                    key,
                    frame_codec,
                    auth_payload_codec,
                )
                .await;

                false
            }
            AuthMessage::AuthValid => {
                info!("Server says auth is valid we can resume");

                true
            }
            unexpected_message => {
                panic!("Unexpected auth message code: {:#?}", unexpected_message)
            }
        }
    } else {
        panic!("Not an auth message {:#?}", frame_message);
    }
}

#[cfg(feature = "quinn")]
async fn ensure_auth(
    client_identifier: ByteStr,
    key: &str,
    connection: &quinn::Connection,
    max_retries: u16,
    frame_codec: FrameCodec,
    auth_payload_codec: &mut AuthPayloadCodec,
) -> Result<bool, quinn::ConnectionError> {
    use futures::TryStreamExt;
    use quinn::VarInt;

    let (mut send, recv) = connection.accept_bi().await?;
    let mut codec_buffer = BytesMut::new();

    let mut framed_reader = FramedRead::new(recv, frame_codec);
    let mut is_first_frame = true;
    let mut retries = 0;

    while let Ok(Some(frame)) = framed_reader.try_next().await {
        let frame_codec = framed_reader.decoder_mut();

        let auth_status = perform_auth(
            &mut codec_buffer,
            &frame.message,
            &mut send,
            client_identifier.clone(),
            key,
            frame_codec,
            auth_payload_codec,
        )
        .await;

        if !auth_status {
            if !is_first_frame {
                if retries > max_retries {
                    return Ok(false);
                }

                info!("Client failed to authorize, retrying?");

                sleep(Duration::from_secs(1)).await;

                retries += 1;
            }

            is_first_frame = false;

            continue;
        } else {
            info!("Server says client is authorized, closing auth stream");

            let mut recv = framed_reader.into_inner();

            let _ = recv.stop(VarInt::from_u32(0));
            let _ = send.finish();

            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(feature = "quinn")]
pub trait ConnectAuthed {
    fn connect_with_authed(
        &self,
        config: quinn::ClientConfig,
        address: SocketAddr,
        server_name: &str,
        client_identifier: ByteStr,
        key: &str,
    ) -> impl std::future::Future<Output = Option<Result<quinn::Connection, quinn::ConnectionError>>>
           + Send;
}

#[cfg(feature = "quinn")]
impl ConnectAuthed for quinn::Endpoint {
    async fn connect_with_authed(
        &self,
        config: quinn::ClientConfig,
        address: SocketAddr,
        server_name: &str,
        client_identifier: ByteStr,
        key: &str,
    ) -> Option<Result<quinn::Connection, quinn::ConnectionError>> {
        match self.connect_with(config, address, server_name) {
            Err(_error) => todo!(),
            Ok(connecting) => match connecting.await {
                Err(_error) => todo!(),
                Ok(connection) => {
                    match ensure_auth(
                        client_identifier,
                        key,
                        &connection,
                        0,
                        FrameCodec::default(),
                        &mut AuthPayloadCodec::default(),
                    )
                    .await
                    {
                        Err(_error) => todo!(),
                        Ok(auth_status) => {
                            if auth_status {
                                Some(Ok(connection))
                            } else {
                                None
                            }
                        }
                    }
                }
            },
        }
    }
}
