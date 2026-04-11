use stargem_server::domain::*;

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
