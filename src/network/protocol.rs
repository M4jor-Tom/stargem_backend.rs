use crate::domain::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "auth_login")]
    AuthLogin { username: String, password: String },

    #[serde(rename = "auth_register")]
    AuthRegister {
        username: String,
        email: String,
        password: String,
    },

    #[serde(rename = "ship_list")]
    ShipList,

    #[serde(rename = "ship_create")]
    ShipCreate { model_id: Uuid, name: String },

    #[serde(rename = "ship_equip_passive")]
    ShipEquipPassive {
        ship_id: Uuid,
        module_id: Uuid,
        slot: usize,
    },

    #[serde(rename = "ship_equip_active")]
    ShipEquipActive {
        ship_id: Uuid,
        module_id: Uuid,
        slot: usize,
    },

    #[serde(rename = "ship_equip_weapon")]
    ShipEquipWeapon { ship_id: Uuid, weapon_id: Uuid },

    #[serde(rename = "hangar_add")]
    HangarAdd { ship_id: Uuid },

    #[serde(rename = "hangar_remove")]
    HangarRemove { ship_id: Uuid },

    #[serde(rename = "hangar_select")]
    HangarSelect { index: usize },

    #[serde(rename = "game_list")]
    GameList { mode: Option<GameMode> },

    #[serde(rename = "game_create")]
    GameCreate {
        mode: GameMode,
        name: String,
        max_players: usize,
    },

    #[serde(rename = "game_join")]
    GameJoin { instance_id: Uuid },

    #[serde(rename = "game_leave")]
    GameLeave { instance_id: Uuid },

    #[serde(rename = "game_start")]
    GameStart { instance_id: Uuid },

    #[serde(rename = "combat_fire")]
    CombatFire { target_id: Uuid, weapon_slot: usize },

    #[serde(rename = "combat_missile")]
    CombatMissile {
        target_id: Uuid,
        missile_slot: usize,
    },

    #[serde(rename = "combat_ability")]
    CombatAbility { ability_key: String },

    #[serde(rename = "movement_update")]
    MovementUpdate {
        position: Position,
        rotation: f32,
        velocity: Position,
    },

    #[serde(rename = "station_dock")]
    StationDock { station_id: Uuid },

    #[serde(rename = "station_undock")]
    StationUndock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "auth_success")]
    AuthSuccess { user_id: Uuid, username: String },

    #[serde(rename = "auth_error")]
    AuthError { message: String },

    #[serde(rename = "ship_list")]
    ShipList { ships: Vec<ShipInfo> },

    #[serde(rename = "ship_created")]
    ShipCreated { ship: ShipInfo },

    #[serde(rename = "ship_updated")]
    ShipUpdated { ship: ShipInfo },

    #[serde(rename = "hangar_updated")]
    HangarUpdated { hangar: HangarInfo },

    #[serde(rename = "game_list")]
    GameList { instances: Vec<GameInfo> },

    #[serde(rename = "game_created")]
    GameCreated { instance: GameInfo },

    #[serde(rename = "game_joined")]
    GameJoined { instance: GameInfo },

    #[serde(rename = "game_left")]
    GameLeft { instance_id: Uuid },

    #[serde(rename = "game_started")]
    GameStarted { instance_id: Uuid },

    #[serde(rename = "game_ended")]
    GameEnded {
        instance_id: Uuid,
        scores: Option<Vec<i32>>,
    },

    #[serde(rename = "player_joined")]
    PlayerJoined { instance_id: Uuid, player_id: Uuid },

    #[serde(rename = "player_left")]
    PlayerLeft { instance_id: Uuid, player_id: Uuid },

    #[serde(rename = "combat_event")]
    CombatEvent { event: CombatEventInfo },

    #[serde(rename = "player_killed")]
    PlayerKilled { killer_id: Uuid, victim_id: Uuid },

    #[serde(rename = "player_respawn")]
    PlayerRespawn { player_id: Uuid, ship_id: Uuid },

    #[serde(rename = "wave_started")]
    WaveStarted { wave: u32 },

    #[serde(rename = "wave_cleared")]
    WaveCleared { wave: u32 },

    #[serde(rename = "player_update")]
    PlayerUpdate {
        player_id: Uuid,
        position: Position,
        rotation: f32,
    },

    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipInfo {
    pub id: Uuid,
    pub model_id: Uuid,
    pub name: String,
    pub passive_modules: Vec<Uuid>,
    pub active_modules: Vec<Uuid>,
    pub weapon_id: Option<Uuid>,
    pub current_shield: f32,
    pub current_armor: f32,
    pub current_energy: f32,
}

impl From<Ship> for ShipInfo {
    fn from(ship: Ship) -> Self {
        ShipInfo {
            id: ship.id,
            model_id: ship.model_id,
            name: ship.name,
            passive_modules: ship.passive_modules,
            active_modules: ship.active_modules,
            weapon_id: ship.weapon_id,
            current_shield: ship.current_shield,
            current_armor: ship.current_armor,
            current_energy: ship.current_energy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HangarInfo {
    pub user_id: Uuid,
    pub ship_ids: Vec<Uuid>,
    pub selected_ship_index: Option<usize>,
}

impl From<Hangar> for HangarInfo {
    fn from(hangar: Hangar) -> Self {
        HangarInfo {
            user_id: hangar.user_id,
            ship_ids: hangar.ship_ids,
            selected_ship_index: hangar.selected_ship_index,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub id: Uuid,
    pub mode: GameMode,
    pub state: GameState,
    pub name: String,
    pub max_players: usize,
    pub current_players: usize,
    pub team_scores: Option<Vec<i32>>,
    pub wave_number: Option<u32>,
}

impl From<GameInstance> for GameInfo {
    fn from(instance: GameInstance) -> Self {
        GameInfo {
            id: instance.id,
            mode: instance.mode,
            state: instance.state,
            name: instance.name,
            max_players: instance.max_players,
            current_players: instance.player_ids.len(),
            team_scores: instance.team_scores,
            wave_number: instance.wave_number,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEventInfo {
    pub id: Uuid,
    pub attacker_id: Uuid,
    pub target_id: Uuid,
    pub event_type: String,
    pub value: f32,
}
