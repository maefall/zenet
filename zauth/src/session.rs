use zwire::session::{ConnectionId, SessionBackend, SessionManager};

#[derive(Debug, Clone, Default)]
pub struct AuthState {
    pub authenticated: bool,
    pub client_id: Option<String>,
    pub authenticated_at: Option<u64>,
}

pub trait AuthSession {
    fn authenticate(&self, connection_id: ConnectionId, client_id: String);
    fn is_authenticated(&self, connection_id: ConnectionId) -> bool;
    fn get_client_id(&self, connection_id: ConnectionId) -> Option<String>;
    fn authenticated_connections(&self) -> Vec<(ConnectionId, String)>;
}

impl<B: SessionBackend> AuthSession for SessionManager<B> {
    fn authenticate(&self, connection_id: ConnectionId, client_id: String) {
        self.with_session_mut(connection_id, |session| {
            if session.get::<AuthState>().is_none() {
                session.insert(AuthState::default());
            }

            if let Some(auth) = session.get_mut::<AuthState>() {
                auth.authenticated = true;
                auth.client_id = Some(client_id);
                auth.authenticated_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );
            }
        });
    }

    fn is_authenticated(&self, connection_id: ConnectionId) -> bool {
        self.with_session(connection_id, |session| {
            session
                .get::<AuthState>()
                .map(|auth| auth.authenticated)
                .unwrap_or(false)
        })
        .unwrap_or(false)
    }

    fn get_client_id(&self, connection_id: ConnectionId) -> Option<String> {
        self.with_session(connection_id, |session| {
            session
                .get::<AuthState>()
                .and_then(|auth| auth.client_id.clone())
        })
        .flatten()
    }

    fn authenticated_connections(&self) -> Vec<(ConnectionId, String)> {
        self.active_connections()
            .into_iter()
            .filter_map(|conn_id| {
                self.with_session(conn_id, |session| {
                    session.get::<AuthState>().and_then(|auth| {
                        if auth.authenticated {
                            auth.client_id.clone().map(|id| (conn_id, id))
                        } else {
                            None
                        }
                    })
                })
                .flatten()
            })
            .collect()
    }
}
