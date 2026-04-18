use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageType {
    Electromagnetic,
    Kinetic,
    Thermic,
}

impl DamageType {
    pub fn effectiveness_against_shield(&self) -> f32 {
        match self {
            DamageType::Electromagnetic => 1.5,
            DamageType::Kinetic => 0.5,
            DamageType::Thermic => 1.0,
        }
    }

    pub fn effectiveness_against_armor(&self) -> f32 {
        match self {
            DamageType::Electromagnetic => 0.5,
            DamageType::Kinetic => 1.5,
            DamageType::Thermic => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipSize {
    Frigate,
    Fighter,
    Interceptor,
}

impl ShipSize {
    pub fn slot_count(&self) -> usize {
        match self {
            ShipSize::Frigate => 6,
            ShipSize::Fighter => 4,
            ShipSize::Interceptor => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrigateRole {
    Engineer,
    LongRange,
    Guard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FighterRole {
    Tackler,
    GunShip,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterceptorRole {
    CoverOps,
    Recon,
    ECM,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShipRole {
    Frigate(FrigateRole),
    Fighter(FighterRole),
    Interceptor(InterceptorRole),
}

impl ShipRole {
    pub fn special_ability_key(&self) -> &'static str {
        match self {
            ShipRole::Frigate(FrigateRole::Engineer) => "Drone",
            ShipRole::Frigate(FrigateRole::LongRange) => "Sniper",
            ShipRole::Frigate(FrigateRole::Guard) => "PhasicShield",
            ShipRole::Fighter(FighterRole::Tackler) => "Cloak",
            ShipRole::Fighter(FighterRole::GunShip) => "Overclock",
            ShipRole::Fighter(FighterRole::Command) => "CommandShield",
            ShipRole::Interceptor(InterceptorRole::CoverOps) => "PlasmaWeb",
            ShipRole::Interceptor(InterceptorRole::Recon) => "HyperPropulsion",
            ShipRole::Interceptor(InterceptorRole::ECM) => "EMSurg",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipStats {
    pub max_shield: f32,
    pub max_armor: f32,
    pub max_energy: f32,
    pub shield_regen: f32,
    pub armor_regen: f32,
    pub energy_regen: f32,
    pub speed: f32,
    pub rotation_speed: f32,
    pub cargo_capacity: f32,
}

impl Default for ShipStats {
    fn default() -> Self {
        Self {
            max_shield: 100.0,
            max_armor: 100.0,
            max_energy: 100.0,
            shield_regen: 5.0,
            armor_regen: 2.0,
            energy_regen: 10.0,
            speed: 100.0,
            rotation_speed: 90.0,
            cargo_capacity: 50.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipModel {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub size: ShipSize,
    pub role: Option<ShipRole>,
    pub base_stats: ShipStats,
    pub price: i64,
    pub passive_module_slots: Vec<PassiveModuleType>,
    pub active_module_count: usize,
    pub weapon_type: WeaponSize,
    pub missile_slots: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponSize {
    Frigate,
    Fighter,
    Interceptor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassiveModuleType {
    Shield,
    Armor,
    Capacitor,
    Motor,
    Computer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveModule {
    pub id: Uuid,
    pub model_id: Uuid,
    pub name: String,
    pub module_type: PassiveModuleType,
    pub stat_modifiers: StatModifiers,
    pub price: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatModifiers {
    pub shield_bonus: f32,
    pub armor_bonus: f32,
    pub energy_bonus: f32,
    pub shield_regen_bonus: f32,
    pub armor_regen_bonus: f32,
    pub energy_regen_bonus: f32,
    pub speed_bonus: f32,
    pub rotation_bonus: f32,
}

impl StatModifiers {
    pub fn apply_to(&self, stats: &mut ShipStats) {
        stats.max_shield += self.shield_bonus;
        stats.max_armor += self.armor_bonus;
        stats.max_energy += self.energy_bonus;
        stats.shield_regen += self.shield_regen_bonus;
        stats.armor_regen += self.armor_regen_bonus;
        stats.energy_regen += self.energy_regen_bonus;
        stats.speed += self.speed_bonus;
        stats.rotation_speed += self.rotation_bonus;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveModule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub energy_cost: f32,
    pub cooldown_ms: i64,
    pub role_restricted: Option<ShipRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    pub id: Uuid,
    pub name: String,
    pub damage: f32,
    pub damage_type: DamageType,
    pub fire_rate: f32,
    pub range: f32,
    pub size: WeaponSize,
    pub heat_per_shot: f32,
    pub max_heat: f32,
    pub cooling_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Missile {
    pub id: Uuid,
    pub name: String,
    pub damage: f32,
    pub damage_type: DamageType,
    pub speed: f32,
    pub tracking: f32,
    pub blast_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ship {
    pub id: Uuid,
    pub user_id: Uuid,
    pub model_id: Uuid,
    pub name: String,
    pub passive_modules: Vec<Uuid>,
    pub active_modules: Vec<Uuid>,
    pub weapon_id: Option<Uuid>,
    pub missiles: Vec<MissileLoadout>,
    pub current_stats: ShipStats,
    pub current_shield: f32,
    pub current_armor: f32,
    pub current_energy: f32,
    pub created_at: DateTime<Utc>,
}

impl Ship {
    pub fn new(user_id: Uuid, model: &ShipModel) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            model_id: model.id,
            name: model.name.clone(),
            passive_modules: Vec::new(),
            active_modules: Vec::new(),
            weapon_id: None,
            missiles: Vec::new(),
            current_stats: model.base_stats.clone(),
            current_shield: model.base_stats.max_shield,
            current_armor: model.base_stats.max_armor,
            current_energy: model.base_stats.max_energy,
            created_at: Utc::now(),
        }
    }

    pub fn new_with_name(user_id: Uuid, model: &ShipModel, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            model_id: model.id,
            name,
            passive_modules: Vec::new(),
            active_modules: Vec::new(),
            weapon_id: None,
            missiles: Vec::new(),
            current_stats: model.base_stats.clone(),
            current_shield: model.base_stats.max_shield,
            current_armor: model.base_stats.max_armor,
            current_energy: model.base_stats.max_energy,
            created_at: Utc::now(),
        }
    }

    pub fn apply_damage(&mut self, damage: f32, damage_type: DamageType) -> DamageResult {
        let shield_effective = damage_type.effectiveness_against_shield();
        let armor_effective = damage_type.effectiveness_against_armor();

        let shield_dmg = damage * shield_effective;

        let (shield_damage, armor_damage) = if shield_dmg > self.current_shield {
            let shield_dmg_dealt = self.current_shield;
            let remaining = shield_dmg - shield_dmg_dealt;
            self.current_shield = 0.0;

            let effective_dmg = remaining * armor_effective;
            let armor_dmg_dealt = effective_dmg.min(self.current_armor);
            self.current_armor -= armor_dmg_dealt;

            (shield_dmg_dealt, armor_dmg_dealt)
        } else {
            self.current_shield -= shield_dmg;
            (shield_dmg, 0.0)
        };

        DamageResult {
            shield_damage,
            armor_damage,
            hull_damage: self.current_armor <= 0.0,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current_armor > 0.0
    }

    pub fn update_stats(&mut self, models: &[ShipModel], passive_modules: &[PassiveModule]) {
        let model = models.iter().find(|m| m.id == self.model_id);
        if let Some(model) = model {
            self.current_stats = model.base_stats.clone();

            for &module_id in &self.passive_modules {
                if let Some(module) = passive_modules.iter().find(|m| m.id == module_id) {
                    module.stat_modifiers.apply_to(&mut self.current_stats);
                }
            }

            self.current_shield = self.current_shield.min(self.current_stats.max_shield);
            self.current_armor = self.current_armor.min(self.current_stats.max_armor);
            self.current_energy = self.current_energy.min(self.current_stats.max_energy);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissileLoadout {
    pub missile_id: Uuid,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageResult {
    pub shield_damage: f32,
    pub armor_damage: f32,
    pub hull_damage: bool,
}

#[cfg(test)]
mod ship_tests {
    use super::*;

    #[test]
    fn test_damage_effectiveness_electromagnetic_vs_shield() {
        let em = DamageType::Electromagnetic;
        assert_eq!(em.effectiveness_against_shield(), 1.5);
        assert_eq!(em.effectiveness_against_armor(), 0.5);
    }

    #[test]
    fn test_damage_effectiveness_kinetic_vs_armor() {
        let kinetic = DamageType::Kinetic;
        assert_eq!(kinetic.effectiveness_against_shield(), 0.5);
        assert_eq!(kinetic.effectiveness_against_armor(), 1.5);
    }

    #[test]
    fn test_damage_effectiveness_thermic_balanced() {
        let thermic = DamageType::Thermic;
        assert_eq!(thermic.effectiveness_against_shield(), 1.0);
        assert_eq!(thermic.effectiveness_against_armor(), 1.0);
    }

    #[test]
    fn test_ship_size_slot_count() {
        assert_eq!(ShipSize::Frigate.slot_count(), 6);
        assert_eq!(ShipSize::Fighter.slot_count(), 4);
        assert_eq!(ShipSize::Interceptor.slot_count(), 3);
    }

    fn create_test_ship() -> Ship {
        Ship {
            id: uuid::Uuid::new_v4(),
            user_id: uuid::Uuid::new_v4(),
            model_id: uuid::Uuid::new_v4(),
            name: "Test Ship".into(),
            passive_modules: Vec::new(),
            active_modules: Vec::new(),
            weapon_id: None,
            missiles: Vec::new(),
            current_stats: ShipStats::default(),
            current_shield: 100.0,
            current_armor: 100.0,
            current_energy: 100.0,
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_ship_apply_damage_electromagnetic() {
        let mut ship = create_test_ship();
        ship.current_shield = 100.0;
        ship.current_armor = 100.0;

        let result = ship.apply_damage(100.0, DamageType::Electromagnetic);

        assert_eq!(result.shield_damage, 100.0);
        assert_eq!(ship.current_shield, 0.0);
        assert!(!result.hull_damage);
    }

    #[test]
    fn test_ship_apply_damage_overflow_to_armor() {
        let mut ship = create_test_ship();
        ship.current_shield = 50.0;
        ship.current_armor = 100.0;

        let result = ship.apply_damage(100.0, DamageType::Electromagnetic);

        assert_eq!(ship.current_shield, 0.0);
        assert!(result.armor_damage > 0.0);
        assert!(!result.hull_damage);
    }

    #[test]
    fn test_ship_apply_damage_kinetic() {
        let mut ship = create_test_ship();
        ship.current_shield = 100.0;
        ship.current_armor = 100.0;

        let result = ship.apply_damage(100.0, DamageType::Kinetic);

        assert_eq!(result.shield_damage, 50.0);
        assert_eq!(ship.current_shield, 50.0);
        assert!(!result.hull_damage);
    }

    #[test]
    fn test_ship_destroyed() {
        let mut ship = create_test_ship();
        ship.current_armor = 50.0;

        let result = ship.apply_damage(500.0, DamageType::Kinetic);

        assert!(result.hull_damage);
        assert!(!ship.is_alive());
    }

    #[test]
    fn test_ship_is_alive() {
        let ship = create_test_ship();
        assert!(ship.is_alive());
    }
}

#[cfg(test)]
mod stats_tests {
    use super::*;

    #[test]
    fn test_stat_modifiers_apply() {
        let mut stats = ShipStats::default();
        let modifiers = StatModifiers {
            shield_bonus: 50.0,
            armor_bonus: 30.0,
            energy_bonus: 20.0,
            speed_bonus: 10.0,
            ..Default::default()
        };

        modifiers.apply_to(&mut stats);

        assert_eq!(stats.max_shield, 150.0);
        assert_eq!(stats.max_armor, 130.0);
        assert_eq!(stats.max_energy, 120.0);
        assert_eq!(stats.speed, 110.0);
    }

    #[test]
    fn test_stat_modifiers_stacking() {
        let mut stats = ShipStats::default();

        let mod1 = StatModifiers {
            shield_bonus: 25.0,
            ..Default::default()
        };
        let mod2 = StatModifiers {
            shield_bonus: 25.0,
            speed_bonus: 50.0,
            ..Default::default()
        };

        mod1.apply_to(&mut stats);
        mod2.apply_to(&mut stats);

        assert_eq!(stats.max_shield, 150.0);
        assert_eq!(stats.speed, 150.0);
    }

    #[test]
    fn test_ship_stats_default() {
        let stats = ShipStats::default();

        assert_eq!(stats.max_shield, 100.0);
        assert_eq!(stats.max_armor, 100.0);
        assert_eq!(stats.max_energy, 100.0);
        assert_eq!(stats.shield_regen, 5.0);
        assert_eq!(stats.armor_regen, 2.0);
        assert_eq!(stats.energy_regen, 10.0);
    }
}
