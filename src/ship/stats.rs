use crate::ship::model::{HullStats, PlayerShip, ShipModel};
use crate::ship::modules::{PassiveModule, PassiveModuleType};

#[derive(Debug, Clone)]
pub struct PlayerShipStats {
    pub max_shield: f32,
    pub max_armor: f32,
    pub max_energy: f32,
    pub speed: f32,
    pub agility: f32,
    pub current_shield: f32,
    pub current_armor: f32,
    pub current_energy: f32,
}

impl PlayerShipStats {
    pub fn compute(
        model: &ShipModel,
        _ship: &PlayerShip,
        passive_modules: &[PassiveModule],
    ) -> Self {
        let mut shield_mult = 1.0f32;
        let mut armor_mult = 1.0f32;
        let mut energy_mult = 1.0f32;
        let mut speed_mult = 1.0f32;
        let mut agility_mult = 1.0f32;

        for module in passive_modules {
            for (stat, value) in &module.stat_modifiers {
                match stat.as_str() {
                    "shield_hp" => shield_mult += value,
                    "armor_hp" => armor_mult += value,
                    "energy" => energy_mult += value,
                    "speed" => speed_mult += value,
                    "agility" => agility_mult += value,
                    _ => {}
                }
            }
        }

        let max_shield = model.base_stats.base_shield * shield_mult;
        let max_armor = model.base_stats.base_armor * armor_mult;
        let max_energy = model.base_stats.base_energy * energy_mult;
        let speed = model.base_stats.base_speed * speed_mult;
        let agility = model.base_stats.base_agility * agility_mult;

        Self {
            max_shield,
            max_armor,
            max_energy,
            speed,
            agility,
            current_shield: max_shield,
            current_armor: max_armor,
            current_energy: max_energy,
        }
    }
}
