mod common;

use serial_test::serial;
use sqlx::PgPool;
use std::sync::Arc;
use stargem_server::db::{PostgresHangarRepository, PostgresShipRepository, PostgresUserRepository};
use stargem_server::domain::{
    DamageType, GameMode, Hangar, PlayerSession, Position, Ship, ShipStats, Weapon,
};
use stargem_server::game::{GameInstanceManager, WeaponShot};

const TEST_SHIP_MODEL_ID: uuid::Uuid = uuid::Uuid::parse_hex("00000000-0000-0000-0000-000000000001").unwrap();
const TEST_WEAPON_ID: uuid::Uuid = uuid::Uuid::parse_hex("00000000-0000-0000-0000-000000000002").unwrap();

fn test_weapon() -> Weapon {
    Weapon {
        id: TEST_WEAPON_ID,
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
        model_id: TEST_SHIP_MODEL_ID,
        name: name.to_string(),
        passive_modules: vec![],
        active_modules: vec![],
        weapon_id: Some(TEST_WEAPON_ID),
        missiles: vec![],
        current_stats: default_ship_stats(),
        current_shield: 100.0,
        current_armor: 100.0,
        current_energy: 100.0,
        created_at: chrono::Utc::now(),
    }
}

fn create_player_session(user_id: uuid::Uuid, ship_id: uuid::Uuid, instance_id: Option<uuid::Uuid>) -> PlayerSession {
    PlayerSession {
        id: uuid::Uuid::new_v4(),
        user_id,
        current_ship_id: ship_id,
        position: Position::default(),
        rotation: 0.0,
        velocity: Position::default(),
        docked_at: None,
        game_instance_id: instance_id,
        created_at: chrono::Utc::now(),
    }
}

#[tokio::test]
#[serial]
async fn test_player_can_destroy_another_ship() {
    let db = common::TestDatabase::new();
    let pool = db.pool().await;

    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let ship_repo = Arc::new(PostgresShipRepository::new(pool.clone()));
    let hangar_repo = Arc::new(PostgresHangarRepository::new(pool.clone()));

    let user1_id = uuid::Uuid::new_v4();
    let user2_id = uuid::Uuid::new_v4();

    let user1 = stargem_server::domain::User {
        id: user1_id,
        username: format!("player1_{}", uuid::Uuid::new_v4()),
        email: format!("p1_{}@test.com", uuid::Uuid::new_v4()),
        password_hash: "$argon2id$v=19$m=19456,t=2,p=1$test".to_string(),
        credits: 1000,
        created_at: chrono::Utc::now(),
        last_login: chrono::Utc::now(),
    };

    let user2 = stargem_server::domain::User {
        id: user2_id,
        username: format!("player2_{}", uuid::Uuid::new_v4()),
        email: format!("p2_{}@test.com", uuid::Uuid::new_v4()),
        password_hash: "$argon2id$v=19$m=19456,t=2,p=1$test".to_string(),
        credits: 1000,
        created_at: chrono::Utc::now(),
        last_login: chrono::Utc::now(),
    };

    user_repo.create(&user1).await.expect("Failed to create user 1");
    user_repo.create(&user2).await.expect("Failed to create user 2");

    let mut ship1 = create_test_ship(user1_id, "Attacker Ship");
    let ship1_id = ship1.id;
    ship_repo.create(&ship1).await.expect("Failed to create ship 1");

    let mut ship2 = create_test_ship(user2_id, "Target Ship");
    let ship2_id = ship2.id;
    ship_repo.create(&ship2).await.expect("Failed to create ship 2");

    let mut hangar1 = Hangar::new(user1_id);
    hangar1.add_ship(ship1_id).expect("Failed to add ship 1 to hangar");
    hangar_repo.add_ship(user1_id, ship1_id).await.expect("Failed to add ship 1 to hangar repo");

    let mut hangar2 = Hangar::new(user2_id);
    hangar2.add_ship(ship2_id).expect("Failed to add ship 2 to hangar");
    hangar_repo.add_ship(user2_id, ship2_id).await.expect("Failed to add ship 2 to hangar repo");

    let mut game_manager = GameInstanceManager::new();

    let instance = game_manager.create_instance(
        "Combat Test Arena".to_string(),
        GameMode::FreeForAll,
        4,
    );

    let instance_id = instance.read().id;

    let player1_session = create_player_session(user1_id, ship1_id, Some(instance_id));
    game_manager.join_instance(instance_id, user1_id, player1_session.clone())
        .expect("Failed to join player 1 to instance");

    let player2_session = create_player_session(user2_id, ship2_id, Some(instance_id));
    game_manager.join_instance(instance_id, user2_id, player2_session.clone())
        .expect("Failed to join player 2 to instance");

    ship1.current_shield = 0.0;
    ship1.current_armor = 100.0;
    game_manager.add_ship_to_instance(instance_id, user1_id, ship1.clone());
    game_manager.add_ship_to_instance(instance_id, user2_id, ship2.clone());

    game_manager.register_player_weapon(instance_id, user1_id, &test_weapon())
        .expect("Failed to register weapon for player 1");

    assert!(!game_manager.is_ship_destroyed(instance_id, user2_id), 
        "Ship 2 should not be destroyed before combat");

    let initial_armor = game_manager.get_ship(instance_id, user2_id)
        .expect("Ship 2 should exist")
        .current_armor;

    assert_eq!(initial_armor, 100.0, "Ship 2 should start with full armor");

    let result = game_manager.fire_player_weapon(instance_id, user1_id, user2_id)
        .expect("Combat should succeed");

    assert_eq!(result.damage_result.armor_damage, 75.0, 
        "Kinetic damage should deal 1.5x (50 * 1.5 = 75) to armor");

    let remaining_armor = game_manager.get_ship(instance_id, user2_id)
        .expect("Ship 2 should still exist")
        .current_armor;

    assert_eq!(remaining_armor, 25.0, "Ship 2 should have 25 armor remaining");

    assert!(!game_manager.is_ship_destroyed(instance_id, user2_id),
        "Ship 2 should not be destroyed yet");

    for _ in 0..10 {
        let result = game_manager.fire_player_weapon(instance_id, user1_id, user2_id);
        if let Ok(res) = result {
            if res.damage_result.hull_damage {
                break;
            }
        }
    }

    assert!(game_manager.is_ship_destroyed(instance_id, user2_id),
        "Ship 2 should be destroyed after enough hits");

    let destroyed_ship = game_manager.get_ship(instance_id, user2_id);
    assert!(destroyed_ship.is_none() || !destroyed_ship.unwrap().is_alive(),
        "Ship 2 should have 0 armor");

    let _ = sqlx::query("DELETE FROM hangars WHERE user_id = ANY($1)")
        .bind(&[user1_id, user2_id])
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM ships WHERE user_id = ANY($1)")
        .bind(&[user1_id, user2_id])
        .execute(&pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = ANY($1)")
        .bind(&[user1_id, user2_id])
        .execute(&pool)
        .await;
}

#[tokio::test]
#[serial]
async fn test_combat_system_weapon_cooldown() {
    let db = common::TestDatabase::new();
    let _pool = db.pool().await;

    let mut game_manager = GameInstanceManager::new();

    let instance = game_manager.create_instance(
        "Cooldown Test".to_string(),
        GameMode::FreeForAll,
        2,
    );
    let instance_id = instance.read().id;

    let user1_id = uuid::Uuid::new_v4();
    let user2_id = uuid::Uuid::new_v4();
    let ship1_id = uuid::Uuid::new_v4();
    let ship2_id = uuid::Uuid::new_v4();

    let ship1 = create_test_ship(user1_id, "Fast Firing Ship");
    let ship2 = create_test_ship(user2_id, "Target Ship");

    game_manager.add_ship_to_instance(instance_id, user1_id, ship1);
    game_manager.add_ship_to_instance(instance_id, user2_id, ship2);

    game_manager.register_player_weapon(instance_id, user1_id, &test_weapon())
        .expect("Failed to register weapon");

    let result1 = game_manager.fire_player_weapon(instance_id, user1_id, user2_id);
    assert!(result1.is_ok(), "First shot should succeed");

    let result2 = game_manager.fire_player_weapon(instance_id, user1_id, user2_id);
    assert!(result2.is_err(), "Second shot should fail due to cooldown");
}

#[tokio::test]
#[serial]
async fn test_combat_events_are_created() {
    let db = common::TestDatabase::new();
    let _pool = db.pool().await;

    let mut game_manager = GameInstanceManager::new();

    let instance = game_manager.create_instance(
        "Event Test".to_string(),
        GameMode::TeamDeathmatch,
        2,
    );
    let instance_id = instance.read().id;

    let user1_id = uuid::Uuid::new_v4();
    let user2_id = uuid::Uuid::new_v4();

    let ship1 = create_test_ship(user1_id, "Event Ship 1");
    let ship2 = create_test_ship(user2_id, "Event Ship 2");

    game_manager.add_ship_to_instance(instance_id, user1_id, ship1);
    game_manager.add_ship_to_instance(instance_id, user2_id, ship2);

    game_manager.register_player_weapon(instance_id, user1_id, &test_weapon())
        .expect("Failed to register weapon");

    let result = game_manager.fire_player_weapon(instance_id, user1_id, user2_id)
        .expect("Combat should succeed");

    assert_eq!(result.event.attacker_id, user1_id);
    assert_eq!(result.event.target_id, user2_id);
    assert_eq!(result.event.instance_id, instance_id);
    assert_eq!(result.event.value, 50.0);
}
