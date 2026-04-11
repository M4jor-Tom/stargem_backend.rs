use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::domain::{
    DamageType, ShipSize, ShipRole, FrigateRole, FighterRole, InterceptorRole,
    ShipStats, WeaponSize, PassiveModuleType, StatModifiers, GameMode, GameState,
    Position,
};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub credits: i64,
    pub created_at: DateTime<Utc>,
    pub last_login: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct HangarRow {
    pub user_id: Uuid,
    pub ship_ids: Vec<Uuid>,
    pub selected_ship_index: Option<i32>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ShipRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub model_id: Uuid,
    pub name: String,
    pub passive_modules: Vec<Uuid>,
    pub active_modules: Vec<Uuid>,
    pub weapon_id: Option<Uuid>,
    pub missiles: serde_json::Value,
    pub current_stats: serde_json::Value,
    pub current_shield: f32,
    pub current_armor: f32,
    pub current_energy: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ShipModelRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub size: String,
    pub role: Option<String>,
    pub base_stats: serde_json::Value,
    pub price: i64,
    pub passive_module_slots: Vec<String>,
    pub active_module_count: i32,
    pub weapon_type: String,
    pub missile_slots: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PassiveModuleRow {
    pub id: Uuid,
    pub model_id: Uuid,
    pub name: String,
    pub module_type: String,
    pub stat_modifiers: serde_json::Value,
    pub price: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ActiveModuleRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub energy_cost: f32,
    pub cooldown_ms: i64,
    pub role_restricted: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WeaponRow {
    pub id: Uuid,
    pub name: String,
    pub damage: f32,
    pub damage_type: String,
    pub fire_rate: f32,
    pub range: f32,
    pub size: String,
    pub heat_per_shot: f32,
    pub max_heat: f32,
    pub cooling_rate: f32,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct MissileRow {
    pub id: Uuid,
    pub name: String,
    pub damage: f32,
    pub damage_type: String,
    pub speed: f32,
    pub tracking: f32,
    pub blast_radius: f32,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct GameInstanceRow {
    pub id: Uuid,
    pub mode: String,
    pub state: String,
    pub name: String,
    pub max_players: i32,
    pub player_ids: Vec<Uuid>,
    pub team_scores: Option<serde_json::Value>,
    pub wave_number: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PlayerSessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub current_ship_id: Uuid,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
    pub rotation: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_z: f32,
    pub docked_at: Option<Uuid>,
    pub game_instance_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SpaceStationRow {
    pub id: Uuid,
    pub name: String,
    pub system_id: Uuid,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
}

pub const MIGRATION_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    credits BIGINT DEFAULT 1000,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS hangars (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    ship_ids UUID[] NOT NULL DEFAULT '{}',
    selected_ship_index INT
);

CREATE TABLE IF NOT EXISTS ship_models (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    size VARCHAR(50) NOT NULL,
    role VARCHAR(50),
    base_stats JSONB NOT NULL,
    price BIGINT NOT NULL,
    passive_module_slots VARCHAR(50)[] NOT NULL,
    active_module_count INT NOT NULL,
    weapon_type VARCHAR(50) NOT NULL,
    missile_slots INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ships (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    model_id UUID NOT NULL REFERENCES ship_models(id),
    name VARCHAR(255) NOT NULL,
    passive_modules UUID[] NOT NULL DEFAULT '{}',
    active_modules UUID[] NOT NULL DEFAULT '{}',
    weapon_id UUID,
    missiles JSONB NOT NULL DEFAULT '[]',
    current_stats JSONB NOT NULL,
    current_shield FLOAT NOT NULL,
    current_armor FLOAT NOT NULL,
    current_energy FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS passive_modules (
    id UUID PRIMARY KEY,
    model_id UUID NOT NULL REFERENCES ship_models(id),
    name VARCHAR(255) NOT NULL,
    module_type VARCHAR(50) NOT NULL,
    stat_modifiers JSONB NOT NULL,
    price BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS active_modules (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    energy_cost FLOAT NOT NULL,
    cooldown_ms BIGINT NOT NULL,
    role_restricted VARCHAR(50)
);

CREATE TABLE IF NOT EXISTS weapons (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    damage FLOAT NOT NULL,
    damage_type VARCHAR(50) NOT NULL,
    fire_rate FLOAT NOT NULL,
    range FLOAT NOT NULL,
    size VARCHAR(50) NOT NULL,
    heat_per_shot FLOAT NOT NULL,
    max_heat FLOAT NOT NULL,
    cooling_rate FLOAT NOT NULL
);

CREATE TABLE IF NOT EXISTS missiles (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    damage FLOAT NOT NULL,
    damage_type VARCHAR(50) NOT NULL,
    speed FLOAT NOT NULL,
    tracking FLOAT NOT NULL,
    blast_radius FLOAT NOT NULL
);

CREATE TABLE IF NOT EXISTS space_stations (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    system_id UUID NOT NULL,
    position_x FLOAT NOT NULL,
    position_y FLOAT NOT NULL,
    position_z FLOAT NOT NULL
);

CREATE TABLE IF NOT EXISTS game_instances (
    id UUID PRIMARY KEY,
    mode VARCHAR(50) NOT NULL,
    state VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    max_players INT NOT NULL,
    player_ids UUID[] NOT NULL DEFAULT '{}',
    team_scores JSONB,
    wave_number INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS player_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    current_ship_id UUID NOT NULL,
    position_x FLOAT NOT NULL,
    position_y FLOAT NOT NULL,
    position_z FLOAT NOT NULL,
    rotation FLOAT NOT NULL,
    velocity_x FLOAT NOT NULL,
    velocity_y FLOAT NOT NULL,
    velocity_z FLOAT NOT NULL,
    docked_at UUID,
    game_instance_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS combat_events (
    id UUID PRIMARY KEY,
    instance_id UUID NOT NULL REFERENCES game_instances(id),
    attacker_id UUID NOT NULL,
    target_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    value FLOAT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ships_user_id ON ships(user_id);
CREATE INDEX idx_ships_model_id ON ships(model_id);
CREATE INDEX idx_game_instances_state ON game_instances(state);
CREATE INDEX idx_game_instances_mode ON game_instances(mode);
CREATE INDEX idx_player_sessions_user_id ON player_sessions(user_id);
CREATE INDEX idx_player_sessions_game_instance_id ON player_sessions(game_instance_id);
"#;
