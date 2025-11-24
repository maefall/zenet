mod mac;

use crate::{storage::AuthStore, AuthPayload};
pub use mac::auth_mac;
use secrecy::ExposeSecret;
use std::time::{SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;
use tokio_util::bytes::Bytes;
use zwire::{Frame, Message};

const DUMMY_KEY: [u8; 32] = [0u8; 32];

pub struct Authenticator<S: AuthStore> {
    store: S,
    skew_seconds: u64,
}

impl<S: AuthStore> Authenticator<S> {
    pub fn new(store: S, skew_seconds: u64) -> Self {
        Self {
            store,
            skew_seconds,
        }
    }

    pub fn process_auth_payload(&self, auth_payload: &AuthPayload) -> Frame {
        let auth_status = self.verify_auth(auth_payload);

        if auth_status {
            Frame {
                message_type: Message::AuthValid,
                payload: Bytes::new(),
            }
        } else {
            Frame {
                message_type: Message::AuthInvalid,
                payload: Bytes::new(),
            }
        }
    }

    pub fn verify_auth(&self, auth: &AuthPayload) -> bool {
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(dur) => dur.as_secs(),
            Err(_) => return false,
        };

        if auth.timestamp > now + self.skew_seconds || now > auth.timestamp + self.skew_seconds {
            return false;
        }

        let key_guard = self.store.get_key(&auth.client_identifier);

        let key_bytes = match key_guard.as_ref() {
            Some(k) => k.expose_secret(),
            None => &DUMMY_KEY,
        };

        let expected_mac = match auth_mac(
            key_bytes,
            &auth.client_identifier,
            auth.timestamp,
            auth.nonce,
        ) {
            Ok(mac) => mac,
            Err(_) => return false,
        };

        let mac_ok: bool = expected_mac.ct_eq(&auth.mac).into();

        if !mac_ok {
            return false;
        }

        if key_guard.is_some() {
            let is_nonce_new = match self.store.insert_nonce(
                &auth.client_identifier,
                auth.nonce,
                auth.timestamp,
                std::time::Duration::from_secs(self.skew_seconds),
            ) {
                Ok(n) => n,
                Err(_) => return false,
            };

            if !is_nonce_new {
                return false;
            }
        }

        true
    }
}
