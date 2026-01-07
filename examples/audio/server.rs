use super::{certificate::load_or_generate_dev_certs, AUTHENTICATOR, SERVER_ADDRESS};
use quinn::{Endpoint, ServerConfig};
use std::{net::SocketAddr, sync::Arc};
use tracing::info;
use zauth::integration::AcceptAuthed;
use zwire::session::{SessionManager, SimpleSessionBackend};

/*
CLIENT CONNECTS TO SERVER & PASSES AUTH:
Client -> Server (bi): RequestAudioTransmission { optional parameters }
Server -> Client (bi): AwaitAudioTransmission { audio_metadata, uni stream stable_id }

Server -> Client (uni): AudioPayload...
*/

pub async fn run() -> anyhow::Result<()> {
    let (_, cert_der, key_der) = load_or_generate_dev_certs()?;

    let server_address: SocketAddr = SERVER_ADDRESS.parse()?;
    let server_config = ServerConfig::with_single_cert(vec![cert_der], key_der)?;

    run_server(server_address, server_config).await
}

async fn run_server(server_address: SocketAddr, server_config: ServerConfig) -> anyhow::Result<()> {
    let endpoint = Endpoint::server(server_config, server_address)?;
    let session_manager = Arc::new(SessionManager::new(SimpleSessionBackend::new()));

    info!("Server listening on {}", server_address);

    while let Ok(Some(connection)) = endpoint
        .accept_authed(session_manager.clone(), AUTHENTICATOR.clone())
        .await
    {
        tokio::spawn(async move {
            info!(
                "Accepted [{}]'s connection request, they passed authorization",
                connection.remote_address(),
            );
        });
    }

    Ok(())
}
