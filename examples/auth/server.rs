use super::{certificate::load_or_generate_dev_certs, AUTHENTICATOR, SERVER_ADDRESS};
use quinn::{Endpoint, ServerConfig};
use std::net::SocketAddr;
use std::sync::Arc;

use tracing::info;
use zauth::integration::AcceptAuthed;

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

use zwire::session::{SessionManager, SimpleSessionBackend};

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

    while let Some(Ok(connection)) = endpoint
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
