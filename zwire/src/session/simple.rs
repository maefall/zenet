use super::{ConnectionId, Session, SessionBackend};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct SimpleSessionBackend {
    sessions: Arc<RwLock<HashMap<ConnectionId, Session>>>,
}

impl SimpleSessionBackend {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for SimpleSessionBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionBackend for SimpleSessionBackend {
    fn create(&self, connection_id: ConnectionId) {
        self.sessions
            .write()
            .unwrap()
            .insert(connection_id, Session::new(connection_id));
    }

    fn remove(&self, connection_id: ConnectionId) {
        self.sessions.write().unwrap().remove(&connection_id);
    }

    fn with_session<F, R>(&self, connection_id: ConnectionId, f: F) -> Option<R>
    where
        F: FnOnce(&Session) -> R,
    {
        self.sessions.read().unwrap().get(&connection_id).map(f)
    }

    fn with_session_mut<F, R>(&self, connection_id: ConnectionId, f: F) -> Option<R>
    where
        F: FnOnce(&mut Session) -> R,
    {
        self.sessions
            .write()
            .unwrap()
            .get_mut(&connection_id)
            .map(f)
    }

    fn active_connections(&self) -> Vec<ConnectionId> {
        self.sessions.read().unwrap().keys().copied().collect()
    }
}
