use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::*;
use crate::error::AppError;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<(), AppError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn update(&self, user: &User) -> Result<(), AppError>;
    async fn update_credits(&self, user_id: Uuid, delta: i64) -> Result<(), AppError>;
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
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT * FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT * FROM users WHERE email = $1"
        )
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
        sqlx::query(
            "UPDATE users SET credits = credits + $2 WHERE id = $1"
        )
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
        let row = sqlx::query_as::<_, ShipRow>(
            "SELECT * FROM ships WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.into()))
    }

    async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<Ship>, AppError> {
        let rows = sqlx::query_as::<_, ShipRow>(
            "SELECT * FROM ships WHERE user_id = $1 ORDER BY created_at"
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
