mod authenticator;
mod codec;
mod storage;

pub use authenticator::Authenticator;
pub use codec::AuthPayloadCodec;
pub use storage::{memory::InMemoryStore, AuthStore, StorageError};

pub mod __zwire_macros_support {
    pub use zwire::__zwire_macros_support::*;
}

use authenticator::auth_mac;
use hmac::digest::InvalidLength;
use rand::Rng;
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};
use zwire::codec::bytes::{ByteStr, Bytes};

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum ZauthError {
    #[error("Invalid key length, {0:?}")]
    #[diagnostic(severity(Error))]
    InvalidKeyLength(#[from] InvalidLength),

    #[error("System clock is either early or late")]
    #[diagnostic(severity(Error))]
    UnsyncClock(#[from] SystemTimeError),
}

#[derive(Debug, Clone)]
pub struct AuthPayload {
    pub client_identifier: ByteStr,
    pub timestamp: u64,
    pub nonce: u128,
    pub mac: Bytes,
}

impl AuthPayload {
    pub fn new(client_identifier: ByteStr, key: &str) -> Result<Self, ZauthError> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let nonce: u128 = rand::rng().random();
        let mac = auth_mac(key.as_bytes(), &client_identifier, timestamp, nonce)?;

        Ok(AuthPayload {
            client_identifier,
            timestamp,
            nonce,
            mac,
        })
    }
}
