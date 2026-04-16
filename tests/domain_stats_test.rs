use stargem_server::domain::*;

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
