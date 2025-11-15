#[cfg(feature = "in_memory")]
pub mod memory;

use secrecy::SecretSlice;
use std::time::Duration;

pub trait AuthStore: Send + Sync {
    fn get_key(&self, client_id: &str) -> Option<SecretSlice<u8>>;

    fn insert_nonce(
        &self,
        client_id: &str,
        nonce: [u8; 16],
        timestamp: u64,
        ttl: Duration,
    ) -> Result<bool, StorageError>;

    fn cleanup(&self) {}
}

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum StorageError {
    #[error("Storage backend failure")]
    #[diagnostic(severity(Error))]
    BackendFailure,
}
