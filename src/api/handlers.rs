use crate::db::{HangarRepository, ShipModelRepository, ShipRepository, UserRepository};
use crate::domain::{Ship, User};
use crate::error::AppError;
use crate::network::session::SessionManager;
use crate::network::{ClientMessage, ServerMessage, ShipInfo};
use crate::security::{
    BruteForceProtector, BruteForceResult, RateLimitResult, RateLimiter, SessionTimeoutManager,
};
use std::sync::Arc;
use uuid::Uuid;

const MAX_REQUESTS_PER_MINUTE: usize = 60;
const BRUTE_FORCE_MAX_ATTEMPTS: usize = 5;
const BRUTE_FORCE_LOCKOUT_SECS: u64 = 300;
const BRUTE_FORCE_RESET_SECS: u64 = 900;
const SESSION_IDLE_TIMEOUT_SECS: u64 = 1800;
const SESSION_ABSOLUTE_TIMEOUT_SECS: u64 = 28800;

pub struct GameService {
    user_repo: Arc<dyn UserRepository>,
    ship_repo: Arc<dyn ShipRepository>,
    ship_model_repo: Arc<dyn ShipModelRepository>,
    hangar_repo: Arc<dyn HangarRepository>,
    session_manager: Arc<SessionManager>,
    rate_limiter: Arc<RateLimiter>,
    brute_force_protector: Arc<BruteForceProtector>,
    session_timeout_manager: Arc<SessionTimeoutManager>,
}

impl GameService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        ship_repo: Arc<dyn ShipRepository>,
        ship_model_repo: Arc<dyn ShipModelRepository>,
        hangar_repo: Arc<dyn HangarRepository>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        Self {
            user_repo,
            ship_repo,
            ship_model_repo,
            hangar_repo,
            session_manager,
            rate_limiter: Arc::new(RateLimiter::new(MAX_REQUESTS_PER_MINUTE, 60)),
            brute_force_protector: Arc::new(BruteForceProtector::new(
                BRUTE_FORCE_MAX_ATTEMPTS,
                BRUTE_FORCE_LOCKOUT_SECS,
                BRUTE_FORCE_RESET_SECS,
            )),
            session_timeout_manager: Arc::new(SessionTimeoutManager::new(
                SESSION_IDLE_TIMEOUT_SECS,
                SESSION_ABSOLUTE_TIMEOUT_SECS,
            )),
        }
    }

    pub async fn handle_message(
        &self,
        session_id: Uuid,
        msg: ClientMessage,
    ) -> Result<Option<ServerMessage>, AppError> {
        if let Some(reason) = self.session_timeout_manager.check_timeout(session_id) {
            tracing::warn!("Session {} timed out: {:?}", session_id, reason);
            return Ok(Some(ServerMessage::Error {
                message: "Session expired, please re-authenticate".into(),
            }));
        }

        match msg {
            ClientMessage::AuthLogin {
                username,
                password,
                ip,
            } => {
                self.handle_login(session_id, username, password, ip.as_deref())
                    .await
            }
            ClientMessage::AuthRegister {
                username,
                email,
                password,
                ip,
            } => {
                self.handle_register(session_id, username, email, password, ip.as_deref())
                    .await
            }
            ClientMessage::ShipList => self.handle_ship_list(session_id).await,
            ClientMessage::ShipCreate { model_id, name } => {
                self.handle_ship_create(session_id, model_id, name).await
            }
            ClientMessage::ShipEquipPassive {
                ship_id,
                module_id,
                slot,
            } => {
                self.handle_ship_equip_passive(session_id, ship_id, module_id, slot)
                    .await
            }
            ClientMessage::ShipEquipActive {
                ship_id,
                module_id,
                slot,
            } => {
                self.handle_ship_equip_active(session_id, ship_id, module_id, slot)
                    .await
            }
            ClientMessage::ShipEquipWeapon { ship_id, weapon_id } => {
                self.handle_ship_equip_weapon(session_id, ship_id, weapon_id)
                    .await
            }
            ClientMessage::HangarAdd { ship_id } => {
                self.handle_hangar_add(session_id, ship_id).await
            }
            ClientMessage::HangarRemove { ship_id } => {
                self.handle_hangar_remove(session_id, ship_id).await
            }
            ClientMessage::HangarSelect { index } => {
                self.handle_hangar_select(session_id, index).await
            }
            _ => Ok(None),
        }
    }

    async fn handle_login(
        &self,
        session_id: Uuid,
        username: String,
        password: String,
        ip: Option<&str>,
    ) -> Result<Option<ServerMessage>, AppError> {
        let rate_key = ip.unwrap_or("unknown");
        match self.rate_limiter.check(rate_key) {
            RateLimitResult::Limited(retry_after) => {
                return Ok(Some(ServerMessage::Error {
                    message: format!("Too many requests, try again in {} seconds", retry_after),
                }));
            }
            RateLimitResult::Allowed => {}
        }

        match self.brute_force_protector.check(rate_key) {
            BruteForceResult::Locked { remaining_secs } => {
                return Ok(Some(ServerMessage::Error {
                    message: format!(
                        "Too many failed attempts, try again in {} seconds",
                        remaining_secs
                    ),
                }));
            }
            BruteForceResult::Allowed => {}
        }

        validate_username(&username)?;
        validate_password(&password)?;

        let user = self
            .user_repo
            .find_by_username(&username)
            .await?
            .ok_or_else(|| AppError::Unauthorized("Invalid credentials".into()))?;

        if !verify_password(&password, &user.password_hash) {
            self.brute_force_protector.record_failure(rate_key);
            return Ok(Some(ServerMessage::AuthError {
                message: "Invalid credentials".into(),
            }));
        }

        self.brute_force_protector.record_success(rate_key);

        if let Some(session) = self.session_manager.get_session(session_id) {
            let mut guard = session.blocking_write();
            guard.user_id = Some(user.id);
        }

        self.session_timeout_manager.touch(session_id);

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
        ip: Option<&str>,
    ) -> Result<Option<ServerMessage>, AppError> {
        let rate_key = ip.unwrap_or("unknown");
        match self.rate_limiter.check(rate_key) {
            RateLimitResult::Limited(retry_after) => {
                return Ok(Some(ServerMessage::Error {
                    message: format!("Too many requests, try again in {} seconds", retry_after),
                }));
            }
            RateLimitResult::Allowed => {}
        }

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
            let mut guard = session.blocking_write();
            guard.user_id = Some(user.id);
        }

        self.session_timeout_manager.touch(session_id);

        Ok(Some(ServerMessage::AuthSuccess {
            user_id: user.id,
            username: user.username,
        }))
    }

    async fn handle_ship_list(&self, session_id: Uuid) -> Result<Option<ServerMessage>, AppError> {
        let user_id = self.get_session_user(session_id)?;
        let ships = self.ship_repo.find_by_user(user_id).await?;

        self.session_timeout_manager.touch(session_id);

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

        if name.len() < 3 || name.len() > 50 {
            return Err(AppError::BadRequest(
                "Ship name must be between 3 and 50 characters".into(),
            ));
        }

        let model = self
            .ship_model_repo
            .find_by_id(model_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Ship model not found".into()))?;

        let ship = Ship::new_with_name(user_id, &model, name);
        self.ship_repo.create(&ship).await?;

        self.session_timeout_manager.touch(session_id);

        Ok(Some(ServerMessage::ShipCreated { ship: ship.into() }))
    }

    async fn handle_ship_equip_passive(
        &self,
        session_id: Uuid,
        ship_id: Uuid,
        _module_id: Uuid,
        _slot: usize,
    ) -> Result<Option<ServerMessage>, AppError> {
        let user_id = self.get_session_user(session_id)?;

        let ship = self
            .ship_repo
            .find_by_id(ship_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Ship not found".into()))?;

        if ship.user_id != user_id {
            return Err(AppError::Forbidden("You do not own this ship".into()));
        }

        self.session_timeout_manager.touch(session_id);

        Ok(Some(ServerMessage::ShipUpdated { ship: ship.into() }))
    }

    async fn handle_ship_equip_active(
        &self,
        session_id: Uuid,
        ship_id: Uuid,
        _module_id: Uuid,
        _slot: usize,
    ) -> Result<Option<ServerMessage>, AppError> {
        let user_id = self.get_session_user(session_id)?;

        let ship = self
            .ship_repo
            .find_by_id(ship_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Ship not found".into()))?;

        if ship.user_id != user_id {
            return Err(AppError::Forbidden("You do not own this ship".into()));
        }

        self.session_timeout_manager.touch(session_id);

        Ok(Some(ServerMessage::ShipUpdated { ship: ship.into() }))
    }

    async fn handle_ship_equip_weapon(
        &self,
        session_id: Uuid,
        ship_id: Uuid,
        _weapon_id: Uuid,
    ) -> Result<Option<ServerMessage>, AppError> {
        let user_id = self.get_session_user(session_id)?;

        let ship = self
            .ship_repo
            .find_by_id(ship_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Ship not found".into()))?;

        if ship.user_id != user_id {
            return Err(AppError::Forbidden("You do not own this ship".into()));
        }

        self.session_timeout_manager.touch(session_id);

        Ok(Some(ServerMessage::ShipUpdated { ship: ship.into() }))
    }

    async fn handle_hangar_add(
        &self,
        session_id: Uuid,
        ship_id: Uuid,
    ) -> Result<Option<ServerMessage>, AppError> {
        let user_id = self.get_session_user(session_id)?;

        let ship = self
            .ship_repo
            .find_by_id(ship_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Ship not found".into()))?;

        if ship.user_id != user_id {
            return Err(AppError::Forbidden("You do not own this ship".into()));
        }

        self.hangar_repo.add_ship(user_id, ship_id).await?;

        let hangar = self
            .hangar_repo
            .get(user_id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to get hangar".into()))?;

        self.session_timeout_manager.touch(session_id);

        Ok(Some(ServerMessage::HangarUpdated {
            hangar: hangar.into(),
        }))
    }

    async fn handle_hangar_remove(
        &self,
        session_id: Uuid,
        ship_id: Uuid,
    ) -> Result<Option<ServerMessage>, AppError> {
        let user_id = self.get_session_user(session_id)?;

        let ship = self
            .ship_repo
            .find_by_id(ship_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Ship not found".into()))?;

        if ship.user_id != user_id {
            return Err(AppError::Forbidden("You do not own this ship".into()));
        }

        self.hangar_repo.remove_ship(user_id, ship_id).await?;

        let hangar = self
            .hangar_repo
            .get(user_id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to get hangar".into()))?;

        self.session_timeout_manager.touch(session_id);

        Ok(Some(ServerMessage::HangarUpdated {
            hangar: hangar.into(),
        }))
    }

    async fn handle_hangar_select(
        &self,
        session_id: Uuid,
        index: usize,
    ) -> Result<Option<ServerMessage>, AppError> {
        let user_id = self.get_session_user(session_id)?;

        let hangar = self
            .hangar_repo
            .get(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Hangar not found".into()))?;

        if index >= hangar.ship_ids.len() {
            return Err(AppError::BadRequest("Invalid ship index".into()));
        }

        self.session_timeout_manager.touch(session_id);

        Ok(Some(ServerMessage::HangarUpdated {
            hangar: hangar.into(),
        }))
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
        return Err(AppError::BadRequest(
            "Username must be at least 3 characters".into(),
        ));
    }
    if username.len() > 32 {
        return Err(AppError::BadRequest(
            "Username must be at most 32 characters".into(),
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::BadRequest(
            "Username can only contain alphanumeric characters, underscores, and hyphens".into(),
        ));
    }
    Ok(())
}

fn validate_email(email: &str) -> Result<(), AppError> {
    if !email.contains('@') || !email.contains('.') {
        return Err(AppError::BadRequest("Invalid email format".into()));
    }
    if email.len() > 255 {
        return Err(AppError::BadRequest(
            "Email must be at most 255 characters".into(),
        ));
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 characters".into(),
        ));
    }
    if password.len() > 128 {
        return Err(AppError::BadRequest(
            "Password must be at most 128 characters".into(),
        ));
    }
    Ok(())
}
