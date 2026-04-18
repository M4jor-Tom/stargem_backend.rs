use crate::domain::{CombatEvent, CombatEventType, DamageResult, DamageType, Ship, Weapon};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

pub struct CombatSystem {
    weapons: HashMap<Uuid, WeaponState>,
}

impl CombatSystem {
    pub fn new() -> Self {
        Self {
            weapons: HashMap::new(),
        }
    }

    pub fn register_weapon(&mut self, ship_id: Uuid, weapon: &Weapon) -> Uuid {
        let id = Uuid::new_v4();
        self.weapons.insert(
            id,
            WeaponState {
                ship_id,
                weapon: weapon.clone(),
                current_heat: 0.0,
                last_fire_time: None,
                overheated: false,
            },
        );
        id
    }

    pub fn fire_weapon(&mut self, weapon_id: Uuid) -> Result<WeaponShot, CombatError> {
        let weapon_state = self
            .weapons
            .get_mut(&weapon_id)
            .ok_or(CombatError::WeaponNotFound)?;

        let now = Utc::now();

        if weapon_state.overheated {
            let cooldown_time = weapon_state
                .last_fire_time
                .map(|t| t + chrono::Duration::seconds(5))
                .unwrap_or(now);

            if now < cooldown_time {
                return Err(CombatError::WeaponOverheated);
            }
            weapon_state.overheated = false;
            weapon_state.current_heat = 0.0;
        }

        let time_since_last = weapon_state
            .last_fire_time
            .map(|t| (now - t).num_milliseconds() as f32 / 1000.0)
            .unwrap_or(weapon_state.weapon.fire_rate);

        let cooldown_ready = time_since_last >= weapon_state.weapon.fire_rate;

        if !cooldown_ready {
            return Err(CombatError::WeaponOnCooldown);
        }

        weapon_state.current_heat += weapon_state.weapon.heat_per_shot;
        weapon_state.last_fire_time = Some(now);

        if weapon_state.current_heat >= weapon_state.weapon.max_heat {
            weapon_state.overheated = true;
        }

        Ok(WeaponShot {
            damage: weapon_state.weapon.damage,
            damage_type: weapon_state.weapon.damage_type,
            heat_generated: weapon_state.weapon.heat_per_shot,
        })
    }

    pub fn update_weapon_cooldowns(&mut self) {
        let now = Utc::now();
        for weapon_state in self.weapons.values_mut() {
            if weapon_state.overheated {
                if let Some(last_time) = weapon_state.last_fire_time {
                    let elapsed = (now - last_time).num_seconds() as f32;
                    let cooldown_time =
                        5.0 + (weapon_state.current_heat / weapon_state.weapon.heat_per_shot) * 2.0;

                    if elapsed >= cooldown_time {
                        weapon_state.overheated = false;
                        weapon_state.current_heat = 0.0;
                    }
                }
            } else if weapon_state.current_heat > 0.0 {
                let last_update = weapon_state.last_fire_time.unwrap_or(now);
                let elapsed = (now - last_update).num_milliseconds() as f32 / 1000.0;
                let heat_dissipated = weapon_state.weapon.cooling_rate * elapsed;
                weapon_state.current_heat = (weapon_state.current_heat - heat_dissipated).max(0.0);
            }
        }
    }

    pub fn apply_damage(ship: &mut Ship, shot: &WeaponShot) -> DamageResult {
        ship.apply_damage(shot.damage, shot.damage_type)
    }

    pub fn create_combat_event(
        &self,
        instance_id: Uuid,
        attacker_id: Uuid,
        target_id: Uuid,
        event_type: CombatEventType,
        value: f32,
    ) -> CombatEvent {
        CombatEvent {
            id: Uuid::new_v4(),
            instance_id,
            attacker_id,
            target_id,
            event_type,
            value,
            timestamp: Utc::now(),
        }
    }
}

impl Default for CombatSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct WeaponState {
    pub ship_id: Uuid,
    pub weapon: Weapon,
    pub current_heat: f32,
    pub last_fire_time: Option<DateTime<Utc>>,
    pub overheated: bool,
}

#[derive(Debug, Clone)]
pub struct WeaponShot {
    pub damage: f32,
    pub damage_type: DamageType,
    pub heat_generated: f32,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum CombatError {
    #[error("Weapon not found")]
    WeaponNotFound,
    #[error("Weapon is on cooldown")]
    WeaponOnCooldown,
    #[error("Weapon is overheated")]
    WeaponOverheated,
    #[error("Invalid target")]
    InvalidTarget,
    #[error("Out of range")]
    OutOfRange,
    #[error("Insufficient energy")]
    InsufficientEnergy,
}

pub struct SpecialAbilityManager {
    active_abilities: HashMap<Uuid, ActiveAbility>,
    cloaked_ships: std::collections::HashSet<Uuid>,
    command_shields: HashMap<Uuid, CommandShieldState>,
}

impl SpecialAbilityManager {
    pub fn new() -> Self {
        Self {
            active_abilities: HashMap::new(),
            cloaked_ships: std::collections::HashSet::new(),
            command_shields: HashMap::new(),
        }
    }

    pub fn activate_drone(&mut self, ship_id: Uuid, target_ship_id: Uuid) {
        self.active_abilities.insert(
            Uuid::new_v4(),
            ActiveAbility {
                ship_id,
                ability_type: AbilityType::Drone,
                start_time: Utc::now(),
                duration_ms: 10000,
                target: Some(target_ship_id),
            },
        );
    }

    pub fn activate_sniper(&mut self, ship_id: Uuid) {
        self.active_abilities.insert(
            Uuid::new_v4(),
            ActiveAbility {
                ship_id,
                ability_type: AbilityType::Sniper,
                start_time: Utc::now(),
                duration_ms: 0,
                target: None,
            },
        );
    }

    pub fn activate_phasic_shield(&mut self, ship_id: Uuid) {
        self.active_abilities.insert(
            Uuid::new_v4(),
            ActiveAbility {
                ship_id,
                ability_type: AbilityType::PhasicShield,
                start_time: Utc::now(),
                duration_ms: 15000,
                target: None,
            },
        );
    }

    pub fn activate_cloak(&mut self, ship_id: Uuid) {
        self.active_abilities.insert(
            Uuid::new_v4(),
            ActiveAbility {
                ship_id,
                ability_type: AbilityType::Cloak,
                start_time: Utc::now(),
                duration_ms: 20000,
                target: None,
            },
        );
        self.cloaked_ships.insert(ship_id);
    }

    pub fn deactivate_cloak(&mut self, ship_id: Uuid) {
        self.cloaked_ships.remove(&ship_id);
        self.active_abilities
            .retain(|_, a| !(a.ship_id == ship_id && matches!(a.ability_type, AbilityType::Cloak)));
    }

    pub fn activate_overclock(&mut self, ship_id: Uuid) {
        self.active_abilities.insert(
            Uuid::new_v4(),
            ActiveAbility {
                ship_id,
                ability_type: AbilityType::Overclock,
                start_time: Utc::now(),
                duration_ms: 10000,
                target: None,
            },
        );
    }

    pub fn activate_command_shield(&mut self, ship_id: Uuid, max_energy: f32) {
        self.command_shields.insert(
            ship_id,
            CommandShieldState {
                remaining_energy: max_energy,
                max_energy,
            },
        );
        self.active_abilities.insert(
            Uuid::new_v4(),
            ActiveAbility {
                ship_id,
                ability_type: AbilityType::CommandShield,
                start_time: Utc::now(),
                duration_ms: 0,
                target: None,
            },
        );
    }

    pub fn activate_plasma_web(&mut self, ship_id: Uuid, target_id: Uuid) {
        self.active_abilities.insert(
            Uuid::new_v4(),
            ActiveAbility {
                ship_id,
                ability_type: AbilityType::PlasmaWeb,
                start_time: Utc::now(),
                duration_ms: 5000,
                target: Some(target_id),
            },
        );
    }

    pub fn activate_hyper_propulsion(&mut self, ship_id: Uuid) {
        self.active_abilities.insert(
            Uuid::new_v4(),
            ActiveAbility {
                ship_id,
                ability_type: AbilityType::HyperPropulsion,
                start_time: Utc::now(),
                duration_ms: 0,
                target: None,
            },
        );
    }

    pub fn activate_em_surge(&mut self, ship_id: Uuid) {
        self.active_abilities.insert(
            Uuid::new_v4(),
            ActiveAbility {
                ship_id,
                ability_type: AbilityType::EMSurg,
                start_time: Utc::now(),
                duration_ms: 3000,
                target: None,
            },
        );
    }

    pub fn is_cloaked(&self, ship_id: Uuid) -> bool {
        self.cloaked_ships.contains(&ship_id)
    }

    pub fn update(&mut self) {
        let now = Utc::now();
        self.active_abilities.retain(|_, ability| {
            if ability.duration_ms == 0 {
                return false;
            }
            let elapsed = (now - ability.start_time).num_milliseconds();
            elapsed < ability.duration_ms
        });

        for ship_id in self.cloaked_ships.clone() {
            if !self
                .active_abilities
                .iter()
                .any(|(_, a)| a.ship_id == ship_id && matches!(a.ability_type, AbilityType::Cloak))
            {
                self.cloaked_ships.remove(&ship_id);
            }
        }
    }

    pub fn handle_damage(&mut self, ship_id: Uuid, damage: f32) -> Option<f32> {
        if let Some(shield) = self.command_shields.get_mut(&ship_id) {
            if shield.remaining_energy > 0.0 {
                shield.remaining_energy -= damage;
                return Some(0.0);
            }
        }

        if self.cloaked_ships.contains(&ship_id) {
            self.deactivate_cloak(ship_id);
        }

        None
    }
}

impl Default for SpecialAbilityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ActiveAbility {
    pub ship_id: Uuid,
    pub ability_type: AbilityType,
    pub start_time: DateTime<Utc>,
    pub duration_ms: i64,
    pub target: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub enum AbilityType {
    Drone,
    Sniper,
    PhasicShield,
    Cloak,
    Overclock,
    CommandShield,
    PlasmaWeb,
    HyperPropulsion,
    EMSurg,
}

#[derive(Debug, Clone)]
pub struct CommandShieldState {
    pub remaining_energy: f32,
    pub max_energy: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DamageType, Ship, Weapon, WeaponSize};

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
            current_stats: Default::default(),
            current_shield: 100.0,
            current_armor: 100.0,
            current_energy: 100.0,
            created_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_combat_system_fire_weapon() {
        let mut combat = CombatSystem::new();
        let weapon = Weapon {
            id: uuid::Uuid::new_v4(),
            name: "Test Weapon".into(),
            damage: 50.0,
            damage_type: DamageType::Kinetic,
            fire_rate: 0.5,
            range: 500.0,
            size: WeaponSize::Fighter,
            heat_per_shot: 10.0,
            max_heat: 100.0,
            cooling_rate: 5.0,
        };

        let weapon_id = combat.register_weapon(uuid::Uuid::new_v4(), &weapon);

        let result = combat.fire_weapon(weapon_id);
        assert!(result.is_ok());

        let shot = result.unwrap();
        assert_eq!(shot.damage, 50.0);
        assert_eq!(shot.damage_type, DamageType::Kinetic);
    }

    #[test]
    fn test_combat_system_overheat() {
        let mut combat = CombatSystem::new();
        let weapon = Weapon {
            id: uuid::Uuid::new_v4(),
            name: "Test Weapon".into(),
            damage: 50.0,
            damage_type: DamageType::Kinetic,
            fire_rate: 0.0,
            range: 500.0,
            size: WeaponSize::Fighter,
            heat_per_shot: 40.0,
            max_heat: 100.0,
            cooling_rate: 1.0,
        };

        let weapon_id = combat.register_weapon(uuid::Uuid::new_v4(), &weapon);

        combat.fire_weapon(weapon_id).unwrap();
        combat.fire_weapon(weapon_id).unwrap();
        combat.fire_weapon(weapon_id).unwrap();
        let result = combat.fire_weapon(weapon_id);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CombatError::WeaponOverheated));
    }

    #[test]
    fn test_combat_system_apply_damage() {
        let mut ship = create_test_ship();
        let shot = WeaponShot {
            damage: 30.0,
            damage_type: DamageType::Thermic,
            heat_generated: 0.0,
        };

        let result = CombatSystem::apply_damage(&mut ship, &shot);

        assert_eq!(result.shield_damage, 30.0);
        assert!(!result.hull_damage);
    }

    #[test]
    fn test_ability_manager_cloak() {
        let mut manager = SpecialAbilityManager::new();
        let ship_id = uuid::Uuid::new_v4();

        manager.activate_cloak(ship_id);

        assert!(manager.is_cloaked(ship_id));

        manager.deactivate_cloak(ship_id);

        assert!(!manager.is_cloaked(ship_id));
    }

    #[test]
    fn test_ability_manager_cloak_breaks_on_damage() {
        let mut manager = SpecialAbilityManager::new();
        let ship_id = uuid::Uuid::new_v4();

        manager.activate_cloak(ship_id);
        assert!(manager.is_cloaked(ship_id));

        let remaining = manager.handle_damage(ship_id, 50.0);

        assert!(remaining.is_none());
        assert!(!manager.is_cloaked(ship_id));
    }

    #[test]
    fn test_ability_manager_command_shield() {
        let mut manager = SpecialAbilityManager::new();
        let ship_id = uuid::Uuid::new_v4();
        let max_energy = 200.0;

        manager.activate_command_shield(ship_id, max_energy);

        let remaining = manager.handle_damage(ship_id, 100.0);

        assert!(remaining.is_some());
        assert_eq!(remaining.unwrap(), 0.0);
    }
}
