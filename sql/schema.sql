CREATE TYPE ship_size AS ENUM ('Frigate', 'Fighter', 'Interceptor');
CREATE TYPE ship_role AS ENUM ('Engineer', 'LongRange', 'Guard', 'Tackler', 'Gunship', 'Command', 'CoverOps', 'Recon', 'Ecm');
CREATE TYPE match_type AS ENUM ('team_deathmatch');
CREATE TYPE match_result AS ENUM ('victory', 'defeat', 'draw');
CREATE TYPE passive_module_type AS ENUM ('shield', 'armor', 'capacitor', 'motor', 'computer');
CREATE TYPE damage_type AS ENUM ('electromagnetic', 'kinetic', 'thermic');
CREATE TYPE weapon_model AS ENUM ('cannon', 'beam', 'lance', 'autocannon', 'pulse', 'railgun', 'machine_gun', 'laser', 'ion_cannon');
CREATE TYPE activation_flow AS ENUM ('ongoing', 'one_shot');

CREATE TABLE users (
    id UUID PRIMARY KEY,
    steam_id TEXT UNIQUE NOT NULL,
    display_name TEXT NOT NULL,
    credit_balance INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE ship_models (
    id UUID PRIMARY KEY,
    size ship_size NOT NULL,
    role ship_role NOT NULL,
    price INTEGER NOT NULL,
    base_shield REAL NOT NULL,
    base_armor REAL NOT NULL,
    base_energy REAL NOT NULL,
    base_speed REAL NOT NULL,
    base_agility REAL NOT NULL,
    shields_count   INTEGER NOT NULL DEFAULT 1 CHECK (shields_count   BETWEEN 0 AND 3),
    armors_count    INTEGER NOT NULL DEFAULT 1 CHECK (armors_count    BETWEEN 0 AND 3),
    capacitors_count INTEGER NOT NULL DEFAULT 1 CHECK (capacitors_count BETWEEN 0 AND 3),
    motors_count    INTEGER NOT NULL DEFAULT 0 CHECK (motors_count    BETWEEN 0 AND 3),
    computers_count INTEGER NOT NULL DEFAULT 1 CHECK (computers_count BETWEEN 0 AND 3)
);

CREATE TABLE passive_modules (
    id UUID PRIMARY KEY,
    model INTEGER NOT NULL,
    module_type passive_module_type NOT NULL,
    shield_hp_modifier  REAL NOT NULL DEFAULT 0.0,
    armor_hp_modifier   REAL NOT NULL DEFAULT 0.0,
    energy_modifier     REAL NOT NULL DEFAULT 0.0,
    speed_modifier      REAL NOT NULL DEFAULT 0.0,
    agility_modifier    REAL NOT NULL DEFAULT 0.0
);

CREATE TABLE active_modules (
    id UUID PRIMARY KEY,
    model INTEGER NOT NULL,
    activation_flow activation_flow NOT NULL,
    energy_cost REAL NOT NULL DEFAULT 0.0,
    cooldown_seconds REAL NOT NULL DEFAULT 0.0
);

CREATE TABLE weapons (
    id UUID PRIMARY KEY,
    name weapon_model NOT NULL,
    size ship_size NOT NULL,
    damage_type damage_type NOT NULL,
    damage_per_shot REAL NOT NULL,
    fire_rate REAL NOT NULL,
    heat_per_shot REAL NOT NULL
);

CREATE TABLE missile_models (
    id UUID PRIMARY KEY,
    damage_type damage_type NOT NULL,
    damage REAL NOT NULL,
    speed REAL NOT NULL,
    turn_rate REAL NOT NULL,
    lifetime_seconds REAL NOT NULL,
    blast_radius REAL NOT NULL DEFAULT 0.0
);

CREATE TABLE player_ships (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    ship_model_id UUID NOT NULL REFERENCES ship_models(id)
);

CREATE TABLE loadout_configs (
    id UUID PRIMARY KEY,
    player_ship_id UUID NOT NULL REFERENCES player_ships(id) ON DELETE CASCADE,
    weapon_id UUID REFERENCES weapons(id),
    missile_id UUID REFERENCES missile_models(id)
);

CREATE TABLE loadout_passive_modules (
    loadout_id UUID NOT NULL REFERENCES loadout_configs(id) ON DELETE CASCADE,
    slot_index INTEGER NOT NULL,
    module_id UUID NOT NULL REFERENCES passive_modules(id),
    PRIMARY KEY (loadout_id, slot_index)
);

CREATE TABLE loadout_active_modules (
    loadout_id UUID NOT NULL REFERENCES loadout_configs(id) ON DELETE CASCADE,
    slot_index INTEGER NOT NULL CHECK (slot_index BETWEEN 0 AND 3),
    module_id UUID NOT NULL REFERENCES active_modules(id),
    PRIMARY KEY (loadout_id, slot_index)
);

CREATE TABLE hangar_assignments (
    user_id UUID NOT NULL REFERENCES users(id),
    slot_index INTEGER NOT NULL CHECK (slot_index BETWEEN 0 AND 3),
    player_ship_id UUID NOT NULL REFERENCES player_ships(id),
    PRIMARY KEY (user_id, slot_index)
);

CREATE TABLE match_records (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    match_type match_type NOT NULL DEFAULT 'team_deathmatch',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    kills INTEGER NOT NULL DEFAULT 0,
    deaths INTEGER NOT NULL DEFAULT 0,
    damage_dealt REAL NOT NULL DEFAULT 0.0,
    damage_taken REAL NOT NULL DEFAULT 0.0,
    result match_result NOT NULL
);
