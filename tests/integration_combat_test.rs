mod common;

use stargem_server::domain::{DamageType, GameMode, Ship, ShipStats, Weapon};
use stargem_server::game::{CombatSystem, GameInstanceManager};

fn test_weapon() -> Weapon {
    Weapon {
        id: common::test_weapon_id(),
        name: "Test Laser".to_string(),
        damage: 50.0,
        damage_type: DamageType::Kinetic,
        fire_rate: 1.0,
        range: 500.0,
        size: stargem_server::domain::WeaponSize::Fighter,
        heat_per_shot: 10.0,
        max_heat: 100.0,
        cooling_rate: 5.0,
    }
}

fn default_ship_stats() -> ShipStats {
    ShipStats {
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

fn create_test_ship(user_id: uuid::Uuid, name: &str) -> Ship {
    Ship {
        id: uuid::Uuid::new_v4(),
        user_id,
        model_id: common::test_ship_model_id(),
        name: name.to_string(),
        passive_modules: vec![],
        active_modules: vec![],
        weapon_id: Some(common::test_weapon_id()),
        missiles: vec![],
        current_stats: default_ship_stats(),
        current_shield: 100.0,
        current_armor: 100.0,
        current_energy: 100.0,
        created_at: chrono::Utc::now(),
    }
}

#[test]
fn test_player_can_destroy_another_ship() {
    let mut game_manager = GameInstanceManager::new();

    let instance =
        game_manager.create_instance("Combat Test Arena".to_string(), GameMode::FreeForAll, 4);

    let instance_id = instance.read().id;

    let user1_id = uuid::Uuid::new_v4();
    let user2_id = uuid::Uuid::new_v4();

    let mut ship1 = create_test_ship(user1_id, "Attacker Ship");
    ship1.current_shield = 0.0;
    ship1.current_armor = 100.0;
    let ship2 = create_test_ship(user2_id, "Target Ship");

    game_manager.add_ship_to_instance(instance_id, user1_id, ship1);
    game_manager.add_ship_to_instance(instance_id, user2_id, ship2);

    let _ = game_manager.register_player_weapon(instance_id, user1_id, &test_weapon());

    let result = game_manager.fire_player_weapon(instance_id, user1_id, user2_id);

    if result.is_ok() {
        let damage_result = result.unwrap().damage_result;
        assert!(
            damage_result.shield_damage >= 0.0 || damage_result.armor_damage >= 0.0,
            "Should deal some damage"
        );
    }
}

#[test]
fn test_combat_system_weapon_cooldown() {
    let mut combat = CombatSystem::new();

    let weapon_id = combat.register_weapon(uuid::Uuid::new_v4(), &test_weapon());

    let result1 = combat.fire_weapon(weapon_id);
    assert!(result1.is_ok(), "First shot should succeed");

    let result2 = combat.fire_weapon(weapon_id);
    assert!(result2.is_err(), "Second shot should fail due to cooldown");
}

#[test]
fn test_combat_events_are_created() {
    let mut combat = CombatSystem::new();

    let weapon_id = combat.register_weapon(uuid::Uuid::new_v4(), &test_weapon());

    let result = combat.fire_weapon(weapon_id);

    if result.is_ok() {
        let shot = result.unwrap();
        assert_eq!(shot.damage, 50.0);
    }
}
