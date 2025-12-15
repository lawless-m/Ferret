use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::types::Session;

pub type SessionManager = Arc<DashMap<Uuid, Session>>;

pub fn create_session_manager() -> SessionManager {
    Arc::new(DashMap::new())
}

pub fn get_or_create_session(manager: &SessionManager, id: Uuid) -> Session {
    manager
        .entry(id)
        .or_insert_with(|| Session::new(id))
        .clone()
}

pub fn update_session(manager: &SessionManager, session: Session) {
    manager.insert(session.id, session);
}

pub fn clear_session(manager: &SessionManager, id: Uuid) {
    if let Some(mut entry) = manager.get_mut(&id) {
        entry.clear();
    }
}

pub fn session_count(manager: &SessionManager) -> usize {
    manager.len()
}
