use super::{
    auth_payload_codec, certificate::load_or_generate_dev_certs, frame_codec, AUTHENTICATOR,
    SERVER_ADDRESS,
};
use futures::StreamExt;
use quinn::{Connection, Endpoint, RecvStream, SendStream, ServerConfig};
use std::{error::Error, net::SocketAddr};
use tokio::io::AsyncWriteExt;
use tokio_util::{
    bytes::BytesMut,
    codec::{Encoder, FramedRead},
};
use tracing::info;
use zauth::AuthMessage;
use zenet::zwire::{DecodeFromFrame, Frame};

/*
CLIENT INITIATES:
Server -> Client (bi): AuthRequired/AuthValid

IF AUTH REQUIRED:
Client -> Server (bi): Auth { auth payload }
Server -> Client (bi): AuthValid/AuthInvalid

IF AUTH VALID:
Client -> Server (bi): RequestAudioTransmission { optional parameters }
Server -> Client (bi): AwaitAudioTransmission { audio_metadata, uni stream stable_id }

Server -> Client (uni): AudioPayload...
*/

use zauth::session::AuthSession;
use zwire::session::{SessionBackend, SessionManager, SimpleSessionBackend};

pub async fn run() -> Result<(), Box<dyn Error>> {
    let (_, cert_der, key_der) = load_or_generate_dev_certs()?;

    let server_address: SocketAddr = SERVER_ADDRESS.parse()?;
    let server_config = ServerConfig::with_single_cert(vec![cert_der], key_der)?;

    let session_manager = SessionManager::new(SimpleSessionBackend::new());

    run_server(server_address, server_config, session_manager).await
}

async fn handle_auth(
    mut send: SendStream,
    receive: RecvStream,
    connection: Connection,
    sessions: SessionManager<impl SessionBackend>,
    codec_buffer: &mut BytesMut,
) -> Result<(), Box<dyn Error>> {
    if !sessions.is_authenticated(connection.stable_id()) {
        let payload = Frame::message_only(AuthMessage::AuthRequired);

        frame_codec().encode(payload, codec_buffer)?;
        send.write_all_buf(codec_buffer).await?;
        codec_buffer.clear();

        let mut framed_reader = FramedRead::new(receive, frame_codec());

        while let Some(Ok(frame)) = framed_reader.next().await {
            if let Some(auth_payload) =
                auth_payload_codec().decode_from_frame(frame, codec_buffer)?
            {
                let (auth_status, auth_response_frame) =
                    AUTHENTICATOR.process_auth_payload(&auth_payload.0);

                if auth_status {
                    sessions.authenticate(
                        connection.stable_id(),
                        auth_payload.0.client_identifier.to_string(),
                    )
                }

                frame_codec().encode(auth_response_frame, codec_buffer)?;
                send.write_all_buf(codec_buffer).await?;
                codec_buffer.clear();
            }
        }
    } else {
        let payload = Frame::message_only(AuthMessage::AuthValid);

        frame_codec().encode(payload, codec_buffer)?;
        send.write_all_buf(codec_buffer).await?;
        codec_buffer.clear();
    }

    Ok(())
}

async fn run_server(
    server_address: SocketAddr,
    server_config: ServerConfig,
    session_manager: SessionManager<impl SessionBackend>,
) -> Result<(), Box<dyn Error>> {
    let endpoint = Endpoint::server(server_config, server_address)?;

    info!("Server listening on {}", server_address);

    while let Some(connecting) = endpoint.accept().await {
        let session_manager = session_manager.clone();

        tokio::spawn(async move {
            if let Ok(connection) = connecting.await {
                info!("Client [{}] connected", connection.remote_address());

                while let Ok((send, receive)) = connection.open_bi().await {
                    let connection = connection.clone();
                    let session_manager = session_manager.clone();

                    let mut codec_write_buffer = BytesMut::new();

                    tokio::spawn(async move {
                        if let Err(e) =
                            handle_auth(send, receive, connection, session_manager, &mut codec_write_buffer).await
                        {
                            tracing::error!("Stream handling failed: {:?}", e);
                        }
                    });
                }
            }
        });
    }

    Ok(())
}
