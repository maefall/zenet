mod authenticator;
mod codec;
mod storage;

pub use authenticator::Authenticator;
pub use codec::AuthPayloadCodec;
pub use storage::{memory::InMemoryStore, AuthStore, StorageError};

use authenticator::auth_mac;
use zwire::codec::bytestring::ByteStr;
use hmac::digest::InvalidLength;
use rand::RngCore;
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};
use tokio_util::bytes::{Bytes, BytesMut, Buf};

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

        let mut nonce_buffer = BytesMut::zeroed(16);

        rand::rng().fill_bytes(&mut nonce_buffer);

        let nonce = nonce_buffer.freeze().get_u128();
        let mac = auth_mac(key.as_bytes(), &client_identifier, timestamp, nonce)?;

        Ok(AuthPayload {
            client_identifier,
            timestamp,
            nonce,
            mac,
        })
    }
}
