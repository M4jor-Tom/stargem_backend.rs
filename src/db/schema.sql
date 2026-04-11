CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    credits BIGINT NOT NULL DEFAULT 1000,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ship_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    size VARCHAR(20) NOT NULL,
    role VARCHAR(30),
    base_stats JSONB NOT NULL,
    price BIGINT NOT NULL,
    passive_module_slots JSONB NOT NULL,
    active_module_count INTEGER NOT NULL DEFAULT 4,
    weapon_type VARCHAR(20) NOT NULL,
    missile_slots INTEGER NOT NULL DEFAULT 2,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    model_id UUID NOT NULL REFERENCES ship_models(id),
    name VARCHAR(100) NOT NULL,
    passive_modules JSONB NOT NULL DEFAULT '[]',
    active_modules JSONB NOT NULL DEFAULT '[]',
    weapon_id UUID,
    missiles JSONB NOT NULL DEFAULT '[]',
    current_stats JSONB NOT NULL,
    current_shield FLOAT NOT NULL,
    current_armor FLOAT NOT NULL,
    current_energy FLOAT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS hangars (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    ship_ids JSONB NOT NULL DEFAULT '[]',
    selected_ship_index INTEGER
);

CREATE TABLE IF NOT EXISTS passive_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id UUID NOT NULL REFERENCES ship_models(id),
    name VARCHAR(100) NOT NULL,
    module_type VARCHAR(30) NOT NULL,
    stat_modifiers JSONB NOT NULL,
    price BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS active_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    energy_cost FLOAT NOT NULL,
    cooldown_ms BIGINT NOT NULL,
    role_restricted VARCHAR(30)
);

CREATE TABLE IF NOT EXISTS weapons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    damage FLOAT NOT NULL,
    damage_type VARCHAR(30) NOT NULL,
    fire_rate FLOAT NOT NULL,
    range FLOAT NOT NULL,
    size VARCHAR(20) NOT NULL,
    heat_per_shot FLOAT NOT NULL,
    max_heat FLOAT NOT NULL,
    cooling_rate FLOAT NOT NULL
);

CREATE TABLE IF NOT EXISTS missiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    damage FLOAT NOT NULL,
    damage_type VARCHAR(30) NOT NULL,
    speed FLOAT NOT NULL,
    tracking FLOAT NOT NULL,
    blast_radius FLOAT NOT NULL
);

CREATE TABLE IF NOT EXISTS space_stations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    system_id UUID NOT NULL,
    position_x FLOAT NOT NULL,
    position_y FLOAT NOT NULL,
    position_z FLOAT NOT NULL
);

CREATE TABLE IF NOT EXISTS game_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mode VARCHAR(30) NOT NULL,
    state VARCHAR(20) NOT NULL,
    name VARCHAR(100) NOT NULL,
    max_players INTEGER NOT NULL,
    player_ids JSONB NOT NULL DEFAULT '[]',
    team_scores JSONB,
    wave_number INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    ended_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS player_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    current_ship_id UUID NOT NULL REFERENCES ships(id),
    position_x FLOAT NOT NULL DEFAULT 0,
    position_y FLOAT NOT NULL DEFAULT 0,
    position_z FLOAT NOT NULL DEFAULT 0,
    rotation FLOAT NOT NULL DEFAULT 0,
    velocity_x FLOAT NOT NULL DEFAULT 0,
    velocity_y FLOAT NOT NULL DEFAULT 0,
    velocity_z FLOAT NOT NULL DEFAULT 0,
    docked_at UUID,
    game_instance_id UUID REFERENCES game_instances(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS combat_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    instance_id UUID NOT NULL REFERENCES game_instances(id),
    attacker_id UUID NOT NULL,
    target_id UUID NOT NULL,
    event_type VARCHAR(30) NOT NULL,
    value FLOAT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ships_user_id ON ships(user_id);
CREATE INDEX idx_ship_models_size ON ship_models(size);
CREATE INDEX idx_game_instances_mode ON game_instances(mode);
CREATE INDEX idx_game_instances_state ON game_instances(state);
CREATE INDEX idx_player_sessions_user_id ON player_sessions(user_id);
CREATE INDEX idx_player_sessions_game_instance ON player_sessions(game_instance_id);
