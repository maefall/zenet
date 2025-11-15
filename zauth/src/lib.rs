mod authenticator;
mod codec;
mod storage;

pub use authenticator::Authenticator;
pub use storage::{memory::InMemoryStore, AuthStore, StorageError};
pub use codec::AuthPayloadCodec;

use authenticator::auth_mac;
use hmac::digest::InvalidLength;
use rand::RngCore;
use std::time::{SystemTime, UNIX_EPOCH, SystemTimeError};

const CLIENT_ID_LENGTH_FIELD_OFFSET: usize = 0;

const FIXED_PART_LENGTH: usize =
    CLIENT_ID_LENGTH_HEADER_LENGTH + TIMESTAMP_LENGTH + NONCE_LENGTH + MAC_LENGTH;
const MAX_CLIENT_IDENTIFIER_LENGTH: usize = u16::MAX as usize;
const CLIENT_ID_LENGTH_HEADER_LENGTH: usize = 2;
const TIMESTAMP_LENGTH: usize = 8;
const NONCE_LENGTH: usize = 16;
const MAC_LENGTH: usize = 32;

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
    pub client_identifier: String,
    pub timestamp: u64,
    pub nonce: [u8; NONCE_LENGTH],
    pub mac: [u8; MAC_LENGTH],
}

impl AuthPayload {
    pub fn new(client_identifier: String, key: &str) -> Result<Self, ZauthError> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let mut nonce = [0u8; 16];

        rand::rng().fill_bytes(&mut nonce);

        let mac = auth_mac(key.as_bytes(), &client_identifier, timestamp, &nonce)?;

        Ok(AuthPayload {
            client_identifier,
            timestamp,
            nonce,
            mac,
        })
    }
}
