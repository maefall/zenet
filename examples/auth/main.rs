mod certificate;
mod client;
mod server;
mod tracing_setup;

use client::run as run_client;
use once_cell::sync::Lazy;
use server::run as run_server;
use std::sync::Arc;
use tracing_setup::setup_tracing;
use zenet::{
    zauth::{AuthPayloadCodec, Authenticator, InMemoryStore},
    zwire::FrameCodec,
};

pub const SERVER_ADDRESS: &str = "127.0.0.1:5000";
pub const CLIENT_IDENTIFIER: &str = "Zeltra-9";
pub const KEY: &str = "";

pub static AUTHENTICATOR: Lazy<Arc<Authenticator<InMemoryStore>>> = Lazy::new(|| {
    let auth_store = InMemoryStore::new(100);

    auth_store.insert_key(CLIENT_IDENTIFIER, KEY.into());

    Arc::new(Authenticator::new(auth_store, 300))
});

static FRAME_CODEC: Lazy<FrameCodec> = Lazy::new(FrameCodec::default);
static AUTH_PAYLOAD_CODEC: Lazy<AuthPayloadCodec> = Lazy::new(AuthPayloadCodec::default);

pub fn frame_codec() -> FrameCodec {
    *FRAME_CODEC
}

pub fn auth_payload_codec() -> AuthPayloadCodec {
    *AUTH_PAYLOAD_CODEC
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "client" {
        run_client().await?
    } else {
        run_server().await?
    }

    Ok(())
}
