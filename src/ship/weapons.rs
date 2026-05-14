use crate::ship::model::ShipSize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponSize {
    Frigate,
    Fighter,
    Interceptor,
}

impl WeaponSize {
    pub fn from_ship_size(size: ShipSize) -> Self {
        match size {
            ShipSize::Frigate => WeaponSize::Frigate,
            ShipSize::Fighter => WeaponSize::Fighter,
            ShipSize::Interceptor => WeaponSize::Interceptor,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    pub id: String,
    pub name: String,
    pub size: WeaponSize,
    pub fire_rate: f32,
    pub damage_per_shot: f32,
    pub heat_per_shot: f32,
    pub max_heat: f32,
    pub cooldown_rate: f32,
}
