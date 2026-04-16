use crate::db::{HangarRepository, ShipRepository, UserRepository};
use crate::domain::{Ship, User};
use crate::error::AppError;
use crate::network::session::SessionManager;
use crate::network::{ClientMessage, ServerMessage, ShipInfo};
use std::sync::Arc;
use uuid::Uuid;

pub struct GameService {
    user_repo: Arc<dyn UserRepository>,
    ship_repo: Arc<dyn ShipRepository>,
    hangar_repo: Arc<dyn HangarRepository>,
    session_manager: Arc<SessionManager>,
}

impl GameService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        ship_repo: Arc<dyn ShipRepository>,
        hangar_repo: Arc<dyn HangarRepository>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        Self {
            user_repo,
            ship_repo,
            hangar_repo,
            session_manager,
        }
    }

    pub async fn handle_message(
        &self,
        session_id: Uuid,
        msg: ClientMessage,
    ) -> Result<Option<ServerMessage>, AppError> {
        match msg {
            ClientMessage::AuthLogin { username, password } => {
                self.handle_login(session_id, username, password).await
            }
            ClientMessage::AuthRegister {
                username,
                email,
                password,
            } => {
                self.handle_register(session_id, username, email, password)
                    .await
            }
            ClientMessage::ShipList => self.handle_ship_list(session_id).await,
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

    async fn handle_login(
        &self,
        session_id: Uuid,
        username: String,
        password: String,
    ) -> Result<Option<ServerMessage>, AppError> {
        validate_username(&username)?;
        validate_password(&password)?;

        let user = self
            .user_repo
            .find_by_username(&username)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;

        if !verify_password(&password, &user.password_hash) {
            return Ok(Some(ServerMessage::AuthError {
                message: "Invalid credentials".into(),
            }));
        }

        if let Some(session) = self.session_manager.get_session(session_id) {
            session.blocking_write().user_id = Some(user.id);
        }

        Ok(Some(ServerMessage::AuthSuccess {
            user_id: user.id,
            username: user.username,
        }))
    }

    async fn handle_register(
        &self,
        session_id: Uuid,
        username: String,
        email: String,
        password: String,
    ) -> Result<Option<ServerMessage>, AppError> {
        validate_username(&username)?;
        validate_email(&email)?;
        validate_password(&password)?;

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

        let password_hash = hash_password(&password)?;
        let user = User::new(username, email, password_hash);
        self.user_repo.create(&user).await?;

        if let Some(session) = self.session_manager.get_session(session_id) {
            session.blocking_write().user_id = Some(user.id);
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

    async fn handle_ship_create(
        &self,
        session_id: Uuid,
        model_id: Uuid,
        name: String,
    ) -> Result<Option<ServerMessage>, AppError> {
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

        Ok(Some(ServerMessage::ShipCreated { ship: ship.into() }))
    }

    async fn handle_hangar_add(
        &self,
        session_id: Uuid,
        ship_id: Uuid,
    ) -> Result<Option<ServerMessage>, AppError> {
        let user_id = self.get_session_user(session_id)?;
        
        let ship = self.ship_repo.find_by_id(ship_id).await?
            .ok_or_else(|| AppError::NotFound("Ship not found".into()))?;
        
        if ship.user_id != user_id {
            return Err(AppError::Unauthorized("Ship does not belong to you".into()));
        }
        
        self.hangar_repo.add_ship(user_id, ship_id).await?;
        
        let hangar = self.hangar_repo.get(user_id).await?
            .ok_or_else(|| AppError::Internal("Failed to get hangar".into()))?;
        
        Ok(Some(ServerMessage::HangarUpdated { hangar: hangar.into() }))
    }

    async fn handle_hangar_remove(
        &self,
        session_id: Uuid,
        ship_id: Uuid,
    ) -> Result<Option<ServerMessage>, AppError> {
        let user_id = self.get_session_user(session_id)?;
        
        self.hangar_repo.remove_ship(user_id, ship_id).await?;
        
        let hangar = self.hangar_repo.get(user_id).await?
            .ok_or_else(|| AppError::Internal("Failed to get hangar".into()))?;
        
        Ok(Some(ServerMessage::HangarUpdated { hangar: hangar.into() }))
    }

    fn get_session_user(&self, session_id: Uuid) -> Result<Uuid, AppError> {
        let session = self
            .session_manager
            .get_session(session_id)
            .ok_or_else(|| AppError::Unauthorized("Session not found".into()))?;
        let guard = session.blocking_read();
        let user_id = guard
            .user_id
            .ok_or_else(|| AppError::Unauthorized("Not authenticated".into()))?;
        Ok(user_id)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))?;
    Ok(hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::{
        password_hash::{PasswordHash, PasswordVerifier},
        Argon2,
    };
    if let Ok(parsed_hash) = PasswordHash::new(hash) {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    } else {
        false
    }
}

fn validate_username(username: &str) -> Result<(), AppError> {
    if username.len() < 3 {
        return Err(AppError::BadRequest("Username must be at least 3 characters".into()));
    }
    if username.len() > 32 {
        return Err(AppError::BadRequest("Username must be at most 32 characters".into()));
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(AppError::BadRequest("Username can only contain alphanumeric characters, underscores, and hyphens".into()));
    }
    Ok(())
}

fn validate_email(email: &str) -> Result<(), AppError> {
    if !email.contains('@') || !email.contains('.') {
        return Err(AppError::BadRequest("Invalid email format".into()));
    }
    if email.len() > 255 {
        return Err(AppError::BadRequest("Email must be at most 255 characters".into()));
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::BadRequest("Password must be at least 8 characters".into()));
    }
    if password.len() > 128 {
        return Err(AppError::BadRequest("Password must be at most 128 characters".into()));
    }
    Ok(())
}
