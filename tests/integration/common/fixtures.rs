use chrono::{DateTime, Utc};
use uuid::Uuid;

pub const TEST_SHIP_MODEL_ID: Uuid =
    Uuid::parse_hex("00000000-0000-0000-0000-000000000001").unwrap();
pub const TEST_WEAPON_ID: Uuid = Uuid::parse_hex("00000000-0000-0000-0000-000000000002").unwrap();

pub struct TestUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub credits: i64,
    pub created_at: DateTime<Utc>,
    pub last_login: DateTime<Utc>,
}

impl TestUser {
    pub fn new(username: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: format!("{}@test.com", username),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$test".to_string(),
            credits: 1000,
            created_at: now,
            last_login: now,
        }
    }

    pub fn with_id(id: Uuid, username: &str) -> Self {
        let now = Utc::now();
        Self {
            id,
            username: username.to_string(),
            email: format!("{}@test.com", username),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$test".to_string(),
            credits: 1000,
            created_at: now,
            last_login: now,
        }
    }
}

pub struct TestShip {
    pub id: Uuid,
    pub user_id: Uuid,
    pub model_id: Uuid,
    pub name: String,
    pub weapon_id: Option<Uuid>,
    pub current_shield: f32,
    pub current_armor: f32,
    pub current_energy: f32,
}

impl TestShip {
    pub fn new(user_id: Uuid, name: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            model_id: TEST_SHIP_MODEL_ID,
            name: name.to_string(),
            weapon_id: Some(TEST_WEAPON_ID),
            current_shield: 100.0,
            current_armor: 100.0,
            current_energy: 100.0,
        }
    }

    pub fn with_low_health(user_id: Uuid, name: &str, health: f32) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            model_id: TEST_SHIP_MODEL_ID,
            name: name.to_string(),
            weapon_id: Some(TEST_WEAPON_ID),
            current_shield: 0.0,
            current_armor: health,
            current_energy: 100.0,
        }
    }

    pub fn with_id(id: Uuid, user_id: Uuid, name: &str) -> Self {
        Self {
            id,
            user_id,
            model_id: TEST_SHIP_MODEL_ID,
            name: name.to_string(),
            weapon_id: Some(TEST_WEAPON_ID),
            current_shield: 100.0,
            current_armor: 100.0,
            current_energy: 100.0,
        }
    }
}

pub struct TestWeapon {
    pub id: Uuid,
    pub name: String,
    pub damage: f32,
    pub damage_type: String,
    pub fire_rate: f32,
}

impl TestWeapon {
    pub fn standard_laser() -> Self {
        Self {
            id: TEST_WEAPON_ID,
            name: "Test Laser".to_string(),
            damage: 50.0,
            damage_type: "Kinetic".to_string(),
            fire_rate: 1.0,
        }
    }

    pub fn high_damage_laser() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "High Damage Laser".to_string(),
            damage: 150.0,
            damage_type: "Kinetic".to_string(),
            fire_rate: 0.5,
        }
    }
}
