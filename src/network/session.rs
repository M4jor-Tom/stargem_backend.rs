use crate::network::ServerMessage;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct ClientSession {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub character_id: Option<Uuid>,
    pub sender: tokio::sync::mpsc::Sender<ServerMessage>,
    pub game_instance_id: Option<Uuid>,
}

impl ClientSession {
    pub fn new(sender: tokio::sync::mpsc::Sender<ServerMessage>) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id: None,
            character_id: None,
            sender,
            game_instance_id: None,
        }
    }

    pub fn get_sender(&self) -> tokio::sync::mpsc::Sender<ServerMessage> {
        self.sender.clone()
    }

    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }
}

pub struct SessionManager {
    sessions: DashMap<Uuid, Arc<RwLock<ClientSession>>>,
    user_to_session: DashMap<Uuid, Uuid>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
            user_to_session: DashMap::new(),
        }
    }

    pub fn add_session(&self, session: Arc<RwLock<ClientSession>>) {
        let id = session.blocking_read().id;
        self.sessions.insert(id, session.clone());
    }

    pub fn remove_session(&self, session_id: Uuid) -> Option<Arc<RwLock<ClientSession>>> {
        let session = self.sessions.remove(&session_id);
        if let Some((_, s)) = &session {
            let user_id = s.blocking_read().user_id;
            if let Some(uid) = user_id {
                self.user_to_session.remove(&uid);
            }
        }
        session.map(|(_, v)| v)
    }

    pub fn get_session(&self, session_id: Uuid) -> Option<Arc<RwLock<ClientSession>>> {
        self.sessions.get(&session_id).map(|r| r.clone())
    }

    pub fn get_session_by_user(&self, user_id: Uuid) -> Option<Arc<RwLock<ClientSession>>> {
        self.user_to_session
            .get(&user_id)
            .and_then(|sid| self.get_session(*sid))
    }

    pub fn set_user_session(&self, user_id: Uuid, session_id: Uuid) {
        self.user_to_session.insert(user_id, session_id);
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn broadcast_to_instance(
        &self,
        instance_id: Uuid,
        msg: ServerMessage,
        except: Option<Uuid>,
    ) {
        for session in self.sessions.iter() {
            let sid = session.key().clone();
            let should_send = except.map_or(true, |e| sid != e);

            if should_send {
                let inst_id = session.blocking_read().game_instance_id;
                if inst_id == Some(instance_id) {
                    let _ = session.blocking_read().sender.try_send(msg.clone());
                }
            }
        }
    }

    pub fn broadcast_all(&self, msg: ServerMessage) {
        for session in self.sessions.iter() {
            let _ = session.blocking_read().sender.try_send(msg.clone());
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}
