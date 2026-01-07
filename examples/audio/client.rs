use super::{certificate::load_or_generate_dev_certs, CLIENT_IDENTIFIER, KEY, SERVER_ADDRESS};
use quinn::{ClientConfig, Endpoint};
use std::{net::SocketAddr, sync::Arc};
use tracing::{error, info};
use zenet::zauth::integration::ConnectAuthed;

const CLIENT_ADDRESS: &str = "127.0.0.1:0";

pub async fn run() -> anyhow::Result<()> {
    let (root_certs, _, _) = load_or_generate_dev_certs()?;

    let client_address: SocketAddr = CLIENT_ADDRESS.parse()?;
    let client_config = quinn::ClientConfig::with_root_certificates(Arc::new(root_certs))?;

    run_client(client_address, client_config).await
}

async fn run_client(client_address: SocketAddr, client_config: ClientConfig) -> anyhow::Result<()> {
    let endpoint = Endpoint::client(client_address)?;

    let server_address: SocketAddr = SERVER_ADDRESS.parse()?;

    let Ok(Some(connection)) = endpoint
        .connect_with_authed(
            client_config,
            server_address,
            "localhost",
            CLIENT_IDENTIFIER.into(),
            KEY,
        )
        .await
    else {
        error!("Failed authorization");

        return Ok(());
    };

    info!(
        "Connected to [{}] and passed authorization",
        connection.remote_address(),
    );

    Ok(())
}
