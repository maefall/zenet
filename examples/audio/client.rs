use super::{
    certificate::load_or_generate_dev_certs, frame_codec, CLIENT_IDENTIFIER, KEY, SERVER_ADDRESS,
};
use futures::TryStreamExt;
use quinn::{ClientConfig, Endpoint};
use std::error::Error;
use std::{net::SocketAddr, sync::Arc};
use tokio_util::{bytes::BytesMut, codec::FramedRead};
use tracing::info;
use zenet::zauth::helpers::handle_auth;

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

    let (mut send, recv) = connection.accept_bi().await?;
    let mut codec_buffer = BytesMut::new();

    let mut framed_reader = FramedRead::new(recv, frame_codec());

    while let Ok(Some(frame)) = framed_reader.try_next().await {
        let auth_status = handle_auth(
            &mut codec_buffer,
            &frame.message,
            &mut send,
            CLIENT_IDENTIFIER.into(),
            KEY,
        )
        .await;

        if !auth_status {
            continue;
        }
    }

    Ok(())
}
