use super::{
    auth_payload_codec, certificate::load_or_generate_dev_certs, frame_codec, CLIENT_IDENTIFIER,
    KEY, SERVER_ADDRESS,
};
use futures::TryStreamExt;
use quinn::{ClientConfig, Endpoint};
use std::error::Error;
use std::{net::SocketAddr, sync::Arc};
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};
use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, Encoder, FramedRead},
};
use tracing::info;
use znet::{
    zauth::AuthPayload,
    zwire::{Frame, MessageType},
};

const CLIENT_ADDRESS: &str = "127.0.0.1:0";

pub async fn run() -> Result<(), Box<dyn Error>> {
    let (root_certs, _, _) = load_or_generate_dev_certs()?;

    let client_address: SocketAddr = CLIENT_ADDRESS.parse()?;
    let client_config = quinn::ClientConfig::with_root_certificates(Arc::new(root_certs))?;

    run_client(client_address, client_config).await
}

async fn run_client(
    client_address: SocketAddr,
    client_config: ClientConfig,
) -> Result<(), Box<dyn Error>> {
    let endpoint = Endpoint::client(client_address)?;

    let server_address: SocketAddr = SERVER_ADDRESS.parse()?;

    let connection = endpoint
        .connect_with(client_config, server_address, "localhost")?
        .await?;

    info!("Connected to [{}] server", server_address);

    let (mut send, recv) = connection.open_bi().await?;

    tokio::spawn(async move {
        let mut codec_buffer = BytesMut::new();

        let mut framed_reader = FramedRead::new(recv, frame_codec());

        while let Ok(Some(frame)) = framed_reader.try_next().await {
            codec_buffer.clear();
            codec_buffer.extend_from_slice(&frame.payload);

            let payload = auth_payload_codec().decode(&mut codec_buffer).unwrap();

            info!("Echoes: Frame: {:?} and Payload: {:?}", frame, payload);
        }
    });

    let mut codec_buffer = BytesMut::new();

    loop {
        let auth_payload = AuthPayload::new(CLIENT_IDENTIFIER.into(), KEY).unwrap();

        auth_payload_codec()
            .encode(auth_payload, &mut codec_buffer)
            .unwrap();

        let frame = Frame {
            message_type: MessageType::Auth,
            payload: codec_buffer.split().freeze(),
        };

        frame_codec().encode(frame, &mut codec_buffer).unwrap();

        send.write_all_buf(&mut codec_buffer).await.unwrap();

        sleep(Duration::from_secs(1)).await;
    }
}
