use stargem_server::domain::{DamageType, Ship, Weapon, WeaponSize};
use stargem_server::game::{CombatError, CombatSystem, SpecialAbilityManager};

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
    let shot = stargem_server::game::WeaponShot {
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
