use crate::domain::*;
use crate::error::AppError;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<(), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn update(&self, user: &User) -> Result<(), AppError>;
    async fn update_credits(&self, user_id: Uuid, delta: i64) -> Result<(), AppError>;
}

#[async_trait]
pub trait ShipModelRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ShipModel>, AppError>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: &User) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, credits, created_at, last_login)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.credits)
        .bind(user.created_at)
        .bind(user.last_login)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn update(&self, user: &User) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users 
            SET username = $2, email = $3, password_hash = $4, credits = $5, last_login = $6
            WHERE id = $1
            "#,
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.credits)
        .bind(user.last_login)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_credits(&self, user_id: Uuid, delta: i64) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET credits = credits + $2 WHERE id = $1")
            .bind(user_id)
            .bind(delta)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct UserRow {
    id: Uuid,
    username: String,
    email: String,
    password_hash: String,
    credits: i64,
    created_at: chrono::DateTime<chrono::Utc>,
    last_login: chrono::DateTime<chrono::Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: row.id,
            username: row.username,
            email: row.email,
            password_hash: row.password_hash,
            credits: row.credits,
            created_at: row.created_at,
            last_login: row.last_login,
        }
    }
}

pub struct PostgresShipModelRepository {
    pool: PgPool,
}

impl PostgresShipModelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ShipModelRepository for PostgresShipModelRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<ShipModel>, AppError> {
        let row = sqlx::query_as::<_, ShipModelRow>("SELECT * FROM ship_models WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.into()))
    }
}

#[derive(sqlx::FromRow)]
struct ShipModelRow {
    id: Uuid,
    name: String,
    description: String,
    size: String,
    role: Option<String>,
    base_stats: sqlx::types::Json<ShipStats>,
    price: i64,
    passive_module_slots: Vec<String>,
    active_module_count: i32,
    weapon_type: String,
    missile_slots: i32,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ShipModelRow> for ShipModel {
    fn from(row: ShipModelRow) -> Self {
        ShipModel {
            id: row.id,
            name: row.name,
            description: row.description,
            size: parse_ship_size(&row.size),
            role: row.role.as_ref().and_then(|r| parse_ship_role(r)),
            base_stats: row.base_stats.0,
            price: row.price,
            passive_module_slots: row
                .passive_module_slots
                .iter()
                .filter_map(|s| parse_passive_module_type(s).ok())
                .collect(),
            active_module_count: row.active_module_count as usize,
            weapon_type: parse_weapon_size(&row.weapon_type),
            missile_slots: row.missile_slots as usize,
            created_at: row.created_at,
        }
    }
}

fn parse_ship_size(s: &str) -> ShipSize {
    match s {
        "Frigate" => ShipSize::Frigate,
        "Fighter" => ShipSize::Fighter,
        "Interceptor" => ShipSize::Interceptor,
        _ => ShipSize::Fighter,
    }
}

fn parse_ship_role(s: &str) -> Option<ShipRole> {
    match s {
        "Engineer" => Some(ShipRole::Frigate(FrigateRole::Engineer)),
        "LongRange" => Some(ShipRole::Frigate(FrigateRole::LongRange)),
        "Guard" => Some(ShipRole::Frigate(FrigateRole::Guard)),
        "Tackler" => Some(ShipRole::Fighter(FighterRole::Tackler)),
        "GunShip" => Some(ShipRole::Fighter(FighterRole::GunShip)),
        "Command" => Some(ShipRole::Fighter(FighterRole::Command)),
        "CoverOps" => Some(ShipRole::Interceptor(InterceptorRole::CoverOps)),
        "Recon" => Some(ShipRole::Interceptor(InterceptorRole::Recon)),
        "ECM" => Some(ShipRole::Interceptor(InterceptorRole::ECM)),
        _ => None,
    }
}

fn parse_weapon_size(s: &str) -> WeaponSize {
    match s {
        "Frigate" => WeaponSize::Frigate,
        "Fighter" => WeaponSize::Fighter,
        "Interceptor" => WeaponSize::Interceptor,
        _ => WeaponSize::Fighter,
    }
}

fn parse_passive_module_type(s: &str) -> Result<PassiveModuleType, ()> {
    match s {
        "Shield" => Ok(PassiveModuleType::Shield),
        "Armor" => Ok(PassiveModuleType::Armor),
        "Capacitor" => Ok(PassiveModuleType::Capacitor),
        "Motor" => Ok(PassiveModuleType::Motor),
        "Computer" => Ok(PassiveModuleType::Computer),
        _ => Err(()),
    }
}

#[async_trait]
pub trait ShipRepository: Send + Sync {
    async fn create(&self, ship: &Ship) -> Result<(), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Ship>, AppError>;
    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Ship>, AppError>;
    async fn update(&self, ship: &Ship) -> Result<(), AppError>;
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;
}

pub struct PostgresShipRepository {
    pool: PgPool,
}

impl PostgresShipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ShipRow {
    id: Uuid,
    user_id: Uuid,
    model_id: Uuid,
    name: String,
    passive_modules: sqlx::types::Json<Vec<Uuid>>,
    active_modules: sqlx::types::Json<Vec<Uuid>>,
    weapon_id: Option<Uuid>,
    missiles: sqlx::types::Json<Vec<MissileLoadout>>,
    current_stats: sqlx::types::Json<ShipStats>,
    current_shield: f32,
    current_armor: f32,
    current_energy: f32,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<ShipRow> for Ship {
    fn from(row: ShipRow) -> Self {
        Ship {
            id: row.id,
            user_id: row.user_id,
            model_id: row.model_id,
            name: row.name,
            passive_modules: row.passive_modules.0,
            active_modules: row.active_modules.0,
            weapon_id: row.weapon_id,
            missiles: row.missiles.0,
            current_stats: row.current_stats.0,
            current_shield: row.current_shield,
            current_armor: row.current_armor,
            current_energy: row.current_energy,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl ShipRepository for PostgresShipRepository {
    async fn create(&self, ship: &Ship) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO ships (id, user_id, model_id, name, passive_modules, active_modules, 
                             weapon_id, missiles, current_stats, current_shield, current_armor, 
                             current_energy, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
        )
        .bind(ship.id)
        .bind(ship.user_id)
        .bind(ship.model_id)
        .bind(&ship.name)
        .bind(sqlx::types::Json(&ship.passive_modules))
        .bind(sqlx::types::Json(&ship.active_modules))
        .bind(ship.weapon_id)
        .bind(sqlx::types::Json(&ship.missiles))
        .bind(sqlx::types::Json(&ship.current_stats))
        .bind(ship.current_shield)
        .bind(ship.current_armor)
        .bind(ship.current_energy)
        .bind(ship.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Ship>, AppError> {
        let row = sqlx::query_as::<_, ShipRow>("SELECT * FROM ships WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Ship>, AppError> {
        let rows = sqlx::query_as::<_, ShipRow>(
            "SELECT * FROM ships WHERE user_id = $1 ORDER BY created_at",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn update(&self, ship: &Ship) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE ships 
            SET name = $2, passive_modules = $3, active_modules = $4, weapon_id = $5,
                missiles = $6, current_stats = $7, current_shield = $8, current_armor = $9,
                current_energy = $10
            WHERE id = $1
            "#,
        )
        .bind(ship.id)
        .bind(&ship.name)
        .bind(sqlx::types::Json(&ship.passive_modules))
        .bind(sqlx::types::Json(&ship.active_modules))
        .bind(ship.weapon_id)
        .bind(sqlx::types::Json(&ship.missiles))
        .bind(sqlx::types::Json(&ship.current_stats))
        .bind(ship.current_shield)
        .bind(ship.current_armor)
        .bind(ship.current_energy)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM ships WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait]
pub trait HangarRepository: Send + Sync {
    async fn get(&self, user_id: Uuid) -> Result<Option<Hangar>, AppError>;
    async fn add_ship(&self, user_id: Uuid, ship_id: Uuid) -> Result<(), AppError>;
    async fn remove_ship(&self, user_id: Uuid, ship_id: Uuid) -> Result<(), AppError>;
    async fn select_ship(&self, user_id: Uuid, index: usize) -> Result<(), AppError>;
}

pub struct PostgresHangarRepository {
    pool: PgPool,
}

impl PostgresHangarRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HangarRepository for PostgresHangarRepository {
    async fn get(&self, user_id: Uuid) -> Result<Option<Hangar>, AppError> {
        let row = sqlx::query_as::<_, HangarRow>(
            "SELECT user_id, ship_ids, selected_ship_index FROM hangars WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| Hangar {
            user_id: r.user_id,
            ship_ids: r.ship_ids,
            selected_ship_index: r.selected_ship_index.map(|i| i as usize),
        }))
    }

    async fn add_ship(&self, user_id: Uuid, ship_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO hangars (user_id, ship_ids, selected_ship_index)
            VALUES ($1, $2::uuid[], 0)
            ON CONFLICT (user_id) DO UPDATE SET ship_ids = array_append(hangars.ship_ids, $2)
            "#,
        )
        .bind(user_id)
        .bind(ship_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn remove_ship(&self, user_id: Uuid, ship_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE hangars 
            SET ship_ids = array_remove(hangars.ship_ids, $2),
                selected_ship_index = NULL
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .bind(ship_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn select_ship(&self, user_id: Uuid, index: usize) -> Result<(), AppError> {
        sqlx::query("UPDATE hangars SET selected_ship_index = $2 WHERE user_id = $1")
            .bind(user_id)
            .bind(index as i32)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct HangarRow {
    user_id: Uuid,
    ship_ids: Vec<Uuid>,
    selected_ship_index: Option<i32>,
}
