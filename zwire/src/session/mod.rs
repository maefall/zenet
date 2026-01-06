mod simple;
pub use simple::SimpleSessionBackend;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub type ConnectionId = usize;

pub struct Session {
    pub connection_id: ConnectionId,
    pub(crate) extensions: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Session {
    pub fn new(connection_id:  ConnectionId) -> Self {
        Self {
            connection_id,
            extensions: HashMap::new(),
        }
    }
    
    pub fn insert<T: Any + Send + Sync>(&mut self, value: T) {
        self.extensions.insert(TypeId::of::<T>(), Box::new(value));
    }
    
    pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.extensions
            .get(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }
    
    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut T> {
        self.extensions
            . get_mut(&TypeId::of::<T>())
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
}

pub trait SessionBackend: Clone + Send + Sync + 'static {
    /// Create a new session
    fn create(&self, connection_id: ConnectionId);
    
    /// Remove a session
    fn remove(&self, connection_id: ConnectionId);
    
    /// Execute with read access to session
    fn with_session<F, R>(&self, connection_id: ConnectionId, f:  F) -> Option<R>
    where
        F: FnOnce(&Session) -> R;
    
    /// Execute with write access to session
    fn with_session_mut<F, R>(&self, connection_id: ConnectionId, f: F) -> Option<R>
    where
        F: FnOnce(&mut Session) -> R;
    
    /// Get all active connection IDs
    fn active_connections(&self) -> Vec<ConnectionId>;
}

#[derive(Clone)]
pub struct SessionManager<B: SessionBackend> {
    backend: B,
}

impl<B: SessionBackend> SessionManager<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }
    
    pub fn create(&self, connection_id: ConnectionId) {
        self.backend.create(connection_id);
    }
    
    pub fn remove(&self, connection_id: ConnectionId) {
        self.backend. remove(connection_id);
    }
    
    pub fn with_session<F, R>(&self, connection_id: ConnectionId, f: F) -> Option<R>
    where
        F: FnOnce(&Session) -> R,
    {
        self.backend.with_session(connection_id, f)
    }
    
    pub fn with_session_mut<F, R>(&self, connection_id:  ConnectionId, f: F) -> Option<R>
    where
        F: FnOnce(&mut Session) -> R,
    {
        self.backend.with_session_mut(connection_id, f)
    }
    
    pub fn active_connections(&self) -> Vec<ConnectionId> {
        self.backend.active_connections()
    }
}
