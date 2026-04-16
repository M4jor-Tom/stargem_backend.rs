-- Initialize test database with schema

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

CREATE INDEX IF NOT EXISTS idx_ships_user_id ON ships(user_id);
CREATE INDEX IF NOT EXISTS idx_ships_model_id ON ships(model_id);
CREATE INDEX IF NOT EXISTS idx_game_instances_state ON game_instances(state);
CREATE INDEX IF NOT EXISTS idx_game_instances_mode ON game_instances(mode);
CREATE INDEX IF NOT EXISTS idx_player_sessions_user_id ON player_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_player_sessions_game_instance_id ON player_sessions(game_instance_id);

-- Seed test data: basic ship model for testing
INSERT INTO ship_models (id, name, description, size, role, base_stats, price, passive_module_slots, active_module_count, weapon_type, missile_slots)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'Test Fighter',
    'A basic test fighter ship',
    'Fighter',
    NULL,
    '{"max_shield": 100.0, "max_armor": 100.0, "max_energy": 100.0, "shield_regen": 5.0, "armor_regen": 2.0, "energy_regen": 10.0, "speed": 100.0, "rotation_speed": 90.0, "cargo_capacity": 50.0}'::jsonb,
    500,
    ARRAY['Shield', 'Armor'],
    2,
    'Fighter',
    2
) ON CONFLICT DO NOTHING;

-- Seed test weapon
INSERT INTO weapons (id, name, damage, damage_type, fire_rate, range, size, heat_per_shot, max_heat, cooling_rate)
VALUES (
    '00000000-0000-0000-0000-000000000002',
    'Test Laser',
    50.0,
    'Kinetic',
    1.0,
    500.0,
    'Fighter',
    10.0,
    100.0,
    5.0
) ON CONFLICT DO NOTHING;
