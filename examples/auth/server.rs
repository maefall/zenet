use super::{
    auth_payload_codec, certificate::load_or_generate_dev_certs, frame_codec, AUTHENTICATOR,
    SERVER_ADDRESS,
};
use futures::StreamExt;
use quinn::{Endpoint, RecvStream, SendStream, ServerConfig};
use std::{error::Error, net::SocketAddr};
use tokio::io::AsyncWriteExt;
use tokio_util::{
    bytes::BytesMut,
    codec::{Encoder, FramedRead},
};
use tracing::info;
use zenet::zwire::{DecodeFromFrame, MessageType};

pub async fn run() -> Result<(), Box<dyn Error>> {
    let (_, cert_der, key_der) = load_or_generate_dev_certs()?;

    let server_address: SocketAddr = SERVER_ADDRESS.parse()?;
    let server_config = ServerConfig::with_single_cert(vec![cert_der], key_der)?;

    run_server(server_address, server_config).await
}

async fn handle_stream(mut send: SendStream, receive: RecvStream) -> Result<(), Box<dyn Error>> {
    let mut framed_reader = FramedRead::new(receive, frame_codec());
    let mut codec_buffer = BytesMut::new();

    while let Some(Ok(frame)) = framed_reader.next().await {
        match frame.message_type {
            MessageType::Auth => {
                if let Some((auth_payload, message_type)) =
                    auth_payload_codec().decode_from_frame(frame, &mut codec_buffer)?
                {
                    tracing::info!("Received auth request ({message_type:?})");

                    let response_frame = AUTHENTICATOR.process_auth_payload(&auth_payload);

                    frame_codec().encode(response_frame, &mut codec_buffer)?;

                    send.write_all_buf(&mut codec_buffer).await?;
                }
            }
            _ => todo!(),
        }
    }

    Ok(())
}

async fn run_server(
    server_address: SocketAddr,
    server_config: ServerConfig,
) -> Result<(), Box<dyn Error>> {
    let endpoint = Endpoint::server(server_config, server_address)?;

    info!("Server listening on {}", server_address);

    while let Some(connecting) = endpoint.accept().await {
        tokio::spawn(async move {
            if let Ok(connection) = connecting.await {
                info!("Client [{}] connected", connection.remote_address());

                while let Ok((send, receive)) = connection.accept_bi().await {
                    tokio::spawn(async move {
                        if let Err(e) = handle_stream(send, receive).await {
                            tracing::error!("Stream handling failed: {:?}", e);
                        }
                    });
                }
            }
        });
    }

    Ok(())
}
