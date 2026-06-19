use crate::ship::model::{PlayerShip, ShipModel};
use crate::ship::modules::PassiveModuleDef;

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
        passive_modules: &[PassiveModuleDef],
    ) -> Self {
        let mut shield_mult = 1.0f32;
        let mut armor_mult = 1.0f32;
        let mut energy_mult = 1.0f32;
        let mut speed_mult = 1.0f32;
        let mut agility_mult = 1.0f32;

        for module in passive_modules {
            shield_mult += module.shield_hp_modifier;
            armor_mult += module.armor_hp_modifier;
            energy_mult += module.energy_modifier;
            speed_mult += module.speed_modifier;
            agility_mult += module.agility_modifier;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ship::model::{HullStats, PlayerShip, ShipModel, ShipRole, ShipSize};
    use crate::ship::modules::{PassiveModuleDef, PassiveModuleType};
    use uuid::Uuid;

    fn dummy_model() -> ShipModel {
        ShipModel {
            id: Uuid::nil(),
            size: ShipSize::Fighter,
            role: ShipRole::Gunship,
            price: 1000,
            base_stats: HullStats {
                base_shield: 100.0, base_armor: 200.0, base_energy: 50.0,
                base_speed: 100.0, base_agility: 20.0,
            },
            shields_count: 1, armors_count: 1, capacitors_count: 1,
            motors_count: 1, computers_count: 1,
        }
    }

    fn dummy_ship() -> PlayerShip {
        PlayerShip {
            id: Uuid::nil(), user_id: Uuid::nil(),
            ship_model_id: Uuid::nil(), loadout_id: None,
        }
    }

    #[test]
    fn test_compute_base_stats_no_modules() {
        let stats = PlayerShipStats::compute(&dummy_model(), &dummy_ship(), &[]);
        assert_eq!(stats.max_shield, 100.0);
        assert_eq!(stats.max_armor, 200.0);
        assert_eq!(stats.max_energy, 50.0);
        assert_eq!(stats.speed, 100.0);
        assert_eq!(stats.agility, 20.0);
        assert_eq!(stats.current_shield, stats.max_shield);
        assert_eq!(stats.current_armor, stats.max_armor);
        assert_eq!(stats.current_energy, stats.max_energy);
    }

    #[test]
    fn test_compute_with_modules_applies_multipliers() {
        let modules = vec![
            PassiveModuleDef {
                id: Uuid::nil(), model: 1,
                module_type: PassiveModuleType::Shield,
                shield_hp_modifier: 0.2, armor_hp_modifier: 0.0,
                energy_modifier: -0.1, speed_modifier: 0.0, agility_modifier: 0.0,
            },
            PassiveModuleDef {
                id: Uuid::nil(), model: 2,
                module_type: PassiveModuleType::Motor,
                shield_hp_modifier: 0.0, armor_hp_modifier: 0.0,
                energy_modifier: 0.0, speed_modifier: 0.15, agility_modifier: -0.05,
            },
        ];
        let stats = PlayerShipStats::compute(&dummy_model(), &dummy_ship(), &modules);
        assert!((stats.max_shield - 120.0).abs() < 1e-4);
        assert!((stats.max_armor - 200.0).abs() < 1e-4);
        assert!((stats.max_energy - 45.0).abs() < 1e-4);
        assert!((stats.speed - 115.0).abs() < 1e-4);
        assert!((stats.agility - 19.0).abs() < 1e-4);
    }

    #[test]
    fn test_compute_with_negative_multipliers() {
        let modules = vec![
            PassiveModuleDef {
                id: Uuid::nil(), model: 1,
                module_type: PassiveModuleType::Computer,
                shield_hp_modifier: -0.5, armor_hp_modifier: -0.5,
                energy_modifier: 0.3, speed_modifier: 0.0, agility_modifier: 0.0,
            },
        ];
        let stats = PlayerShipStats::compute(&dummy_model(), &dummy_ship(), &modules);
        assert!((stats.max_shield - 50.0).abs() < 1e-6);
        assert!((stats.max_armor - 100.0).abs() < 1e-6);
        assert!((stats.max_energy - 65.0).abs() < 1e-6);
    }
}
