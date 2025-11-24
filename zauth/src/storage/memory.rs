use super::StorageError;
use dashmap::{mapref::entry::Entry, DashMap};
use secrecy::SecretSlice;
use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
    time::Duration,
};

type Nonce = u128;
type Timestamp = u64;
type NonceEntry = (Nonce, Timestamp);
type NonceDeque = VecDeque<NonceEntry>;
type NonceSet = HashSet<Nonce>;
type ClientNonces = (NonceDeque, NonceSet);

pub struct InMemoryStore {
    keys: Arc<DashMap<String, SecretSlice<u8>>>,
    nonces: Arc<DashMap<String, ClientNonces>>,
    max_per_client: usize,
}

impl InMemoryStore {
    pub fn new(max_per_client: usize) -> Self {
        Self {
            keys: Arc::new(DashMap::new()),
            nonces: Arc::new(DashMap::new()),
            max_per_client,
        }
    }

    pub fn insert_key(&self, client_id: &str, key: Vec<u8>) {
        self.keys.insert(client_id.to_string(), key.into());
    }
}

impl super::AuthStore for InMemoryStore {
    fn get_key(&self, client_id: &str) -> Option<SecretSlice<u8>> {
        self.keys.get(client_id).map(|v| v.clone())
    }

    fn insert_nonce(
        &self,
        client_id: &str,
        nonce: u128,
        timestamp: u64,
        ttl: Duration,
    ) -> Result<bool, StorageError> {
        let cutoff = timestamp.saturating_sub(ttl.as_secs());

        match self.nonces.entry(client_id.to_string()) {
            Entry::Occupied(mut entry) => {
                let (deque, set) = entry.get_mut();

                while let Some((_front_nonce, front_ts)) = deque.front() {
                    if *front_ts < cutoff {
                        if let Some((rnonce, _)) = deque.pop_front() {
                            set.remove(&rnonce);
                        }
                    } else {
                        break;
                    }
                }

                if set.contains(&nonce) {
                    return Ok(false);
                }

                #[allow(clippy::collapsible_if)]
                if deque.len() >= self.max_per_client {
                    if let Some((rnonce, _)) = deque.pop_front() {
                        set.remove(&rnonce);
                    }
                }

                deque.push_back((nonce, timestamp));
                set.insert(nonce);
                Ok(true)
            }
            Entry::Vacant(entry) => {
                let mut deque = VecDeque::new();
                let mut set = HashSet::new();
                deque.push_back((nonce, timestamp));
                set.insert(nonce);
                entry.insert((deque, set));
                Ok(true)
            }
        }
    }
}
