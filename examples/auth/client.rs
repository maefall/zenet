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
    codec::{Encoder, FramedRead},
};
use tracing::info;
use zauth::AuthMessage;
use zenet::{zauth::AuthPayload, zwire::EncodeIntoFrame};

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
        let mut framed_reader = FramedRead::new(recv, frame_codec());

        while let Ok(Some(frame)) = framed_reader.try_next().await {
            info!("{:?}", AuthMessage::try_from(&frame.message).unwrap());
        }
    });

    let mut codec_buffer = BytesMut::new();

    loop {
        let auth_payload = AuthPayload::new(CLIENT_IDENTIFIER.into(), KEY).unwrap();
        let frame = auth_payload_codec().encode_into_frame(
            auth_payload,
            AuthMessage::Auth,
            &mut codec_buffer,
        )?;

        frame_codec().encode(frame, &mut codec_buffer).unwrap();

        send.write_all_buf(&mut codec_buffer).await.unwrap();

        sleep(Duration::from_secs(1)).await;
    }
}
