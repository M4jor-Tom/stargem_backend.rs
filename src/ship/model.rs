use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShipSize {
    Frigate,
    Fighter,
    Interceptor,
}

impl ShipSize {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "frigate" => Some(Self::Frigate),
            "fighter" => Some(Self::Fighter),
            "interceptor" => Some(Self::Interceptor),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShipRole {
    Engineer,
    LongRange,
    Guard,
    Tackler,
    Gunship,
    Command,
    CoverOps,
    Recon,
    Ecm,
}

impl ShipRole {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "engineer" => Some(Self::Engineer),
            "longrange" => Some(Self::LongRange),
            "guard" => Some(Self::Guard),
            "tackler" => Some(Self::Tackler),
            "gunship" => Some(Self::Gunship),
            "command" => Some(Self::Command),
            "coverops" => Some(Self::CoverOps),
            "recon" => Some(Self::Recon),
            "ecm" => Some(Self::Ecm),
            _ => None,
        }
    }

    pub fn special_module(&self) -> &'static str {
        match self {
            Self::Engineer => "drones",
            Self::LongRange => "sniper_weapon",
            Self::Guard => "phasic_shield",
            Self::Tackler => "cloak",
            Self::Gunship => "overclock",
            Self::Command => "command_shield",
            Self::CoverOps => "plasma_web",
            Self::Recon => "hyper_propulsion",
            Self::Ecm => "electromagnetic_surge",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HullStats {
    pub base_shield: f32,
    pub base_armor: f32,
    pub base_energy: f32,
    pub base_speed: f32,
    pub base_agility: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipModel {
    pub id: String,
    pub name: String,
    pub size: ShipSize,
    pub role: ShipRole,
    pub price: i32,
    pub base_stats: HullStats,
    pub passive_slots_layout: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PlayerShip {
    pub id: String,
    pub user_id: String,
    pub ship_model_id: String,
    pub loadout_id: Option<String>,
}
