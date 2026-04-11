use std::sync::Arc;
use uuid::Uuid;
use crate::domain::{User, Ship, GameMode};
use crate::db::{UserRepository, ShipRepository};
use crate::error::AppError;
use crate::network::{ClientMessage, ServerMessage, ShipInfo};
use crate::network::session::SessionManager;

pub struct GameService {
    user_repo: Arc<dyn UserRepository>,
    ship_repo: Arc<dyn ShipRepository>,
    session_manager: Arc<SessionManager>,
}

impl GameService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        ship_repo: Arc<dyn ShipRepository>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        Self {
            user_repo,
            ship_repo,
            session_manager,
        }
    }

    pub async fn handle_message(&self, session_id: Uuid, msg: ClientMessage) -> Result<Option<ServerMessage>, AppError> {
        match msg {
            ClientMessage::AuthLogin { username, password } => {
                self.handle_login(session_id, username, password).await
            }
            ClientMessage::AuthRegister { username, email, password } => {
                self.handle_register(session_id, username, email, password).await
            }
            ClientMessage::ShipList => {
                self.handle_ship_list(session_id).await
            }
            ClientMessage::ShipCreate { model_id, name } => {
                self.handle_ship_create(session_id, model_id, name).await
            }
            ClientMessage::HangarAdd { ship_id } => {
                self.handle_hangar_add(session_id, ship_id).await
            }
            ClientMessage::HangarRemove { ship_id } => {
                self.handle_hangar_remove(session_id, ship_id).await
            }
            _ => Ok(None),
        }
    }

    async fn handle_login(&self, session_id: Uuid, username: String, password: String) -> Result<Option<ServerMessage>, AppError> {
        let user = self.user_repo.find_by_username(&username).await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;
        
        if !verify_password(&password, &user.password_hash) {
            return Ok(Some(ServerMessage::AuthError {
                message: "Invalid credentials".into(),
            }));
        }

        if let Some(session) = self.session_manager.get_session(session_id) {
            session.write().user_id = Some(user.id);
        }

        Ok(Some(ServerMessage::AuthSuccess {
            user_id: user.id,
            username: user.username,
        }))
    }

    async fn handle_register(&self, session_id: Uuid, username: String, email: String, password: String) -> Result<Option<ServerMessage>, AppError> {
        if self.user_repo.find_by_username(&username).await?.is_some() {
            return Ok(Some(ServerMessage::AuthError {
                message: "Username already taken".into(),
            }));
        }

        if self.user_repo.find_by_email(&email).await?.is_some() {
            return Ok(Some(ServerMessage::AuthError {
                message: "Email already registered".into(),
            }));
        }

        let password_hash = hash_password(&password);
        let user = User::new(username, email, password_hash);
        self.user_repo.create(&user).await?;

        if let Some(session) = self.session_manager.get_session(session_id) {
            session.write().user_id = Some(user.id);
        }

        Ok(Some(ServerMessage::AuthSuccess {
            user_id: user.id,
            username: user.username,
        }))
    }

    async fn handle_ship_list(&self, session_id: Uuid) -> Result<Option<ServerMessage>, AppError> {
        let user_id = self.get_session_user(session_id)?;
        let ships = self.ship_repo.find_by_user(user_id).await?;
        
        let ship_infos: Vec<ShipInfo> = ships.into_iter().map(|s| s.into()).collect();
        
        Ok(Some(ServerMessage::ShipList { ships: ship_infos }))
    }

    async fn handle_ship_create(&self, session_id: Uuid, model_id: Uuid, name: String) -> Result<Option<ServerMessage>, AppError> {
        let user_id = self.get_session_user(session_id)?;
        
        let ship = Ship {
            id: Uuid::new_v4(),
            user_id,
            model_id,
            name,
            passive_modules: Vec::new(),
            active_modules: Vec::new(),
            weapon_id: None,
            missiles: Vec::new(),
            current_stats: Default::default(),
            current_shield: 100.0,
            current_armor: 100.0,
            current_energy: 100.0,
            created_at: chrono::Utc::now(),
        };
        
        self.ship_repo.create(&ship).await?;
        
        Ok(Some(ServerMessage::ShipCreated {
            ship: ship.into(),
        }))
    }

    async fn handle_hangar_add(&self, session_id: Uuid, ship_id: Uuid) -> Result<Option<ServerMessage>, AppError> {
        Ok(None)
    }

    async fn handle_hangar_remove(&self, session_id: Uuid, ship_id: Uuid) -> Result<Option<ServerMessage>, AppError> {
        Ok(None)
    }

    fn get_session_user(&self, session_id: Uuid) -> Result<Uuid, AppError> {
        let session = self.session_manager.get_session(session_id)
            .ok_or_else(|| AppError::Unauthorized("Session not found".into()))?;
        let guard = session.read();
        let user_id = guard.user_id
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".into()))?;
        Ok(user_id)
    }
}

fn hash_password(password: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    format!("sha256:{:x}", hasher.finish())
}

fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}
