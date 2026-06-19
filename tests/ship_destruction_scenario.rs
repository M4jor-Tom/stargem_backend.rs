use stargem_backend::combat::damage::{apply_damage, DamageMultipliers, DamageType};
use stargem_backend::game_mode::{GameMode, TeamDeathmatch};
use uuid::Uuid;

#[test]
fn gunship_cannon_destroys_recon_in_team_deathmatch() {
    let mult = DamageMultipliers::default();

    let attacker = Uuid::from_u128(1);
    let defender = Uuid::from_u128(2);

    let mut match_ = TeamDeathmatch::new(vec![attacker, defender], 50, 600.0);

    let mut shield = 90.0_f32;
    let mut armor = 70.0_f32;
    let dmg = 25.0_f32;
    let dtype = DamageType::Kinetic;

    for shot in 1..=7 {
        let r = apply_damage(dtype, dmg, shield, armor, &mult);
        shield = r.shield_remaining;
        armor = r.armor_remaining;
        assert!(!r.ship_destroyed, "shot {shot} should not destroy");
    }
    assert!((shield - 2.5).abs() < 1e-4, "shield should be 2.5 after 7 shots");
    assert!((armor - 70.0).abs() < 1e-4, "armor should be untouched");

    let r = apply_damage(dtype, dmg, shield, armor, &mult);
    assert_eq!(r.shield_remaining, 0.0);
    assert!((r.armor_remaining - 55.0).abs() < 1e-4);
    assert!(!r.ship_destroyed);
    shield = r.shield_remaining;
    armor = r.armor_remaining;

    for shot in 9..=10 {
        let r = apply_damage(dtype, dmg, shield, armor, &mult);
        shield = r.shield_remaining;
        armor = r.armor_remaining;
        assert!(!r.ship_destroyed, "shot {shot} should not destroy");
    }
    assert!((armor - 17.5).abs() < 1e-4, "armor should be 17.5 after 10 shots");

    let r = apply_damage(dtype, dmg, shield, armor, &mult);
    assert!(r.ship_destroyed, "shot 11 should destroy");
    assert_eq!(r.armor_remaining, 0.0);

    assert!(match_.results().is_none());
    assert!(!match_.is_finished());

    for _ in 0..50 {
        match_.on_player_death(defender, Some(attacker));
    }

    assert!(match_.is_finished());
    let results = match_.results().unwrap();
    assert_eq!(results.reason, "score_limit");
    assert_eq!(results.winning_team, Some(0));

    let a_stats = results.players.iter().find(|p| p.player_id == attacker).unwrap();
    let d_stats = results.players.iter().find(|p| p.player_id == defender).unwrap();
    assert_eq!(a_stats.kills, 50);
    assert_eq!(d_stats.deaths, 50);
    assert_eq!(results.team_scores[&0], 50);
}

#[test]
fn electromag_pierces_shield_quickly() {
    let mult = DamageMultipliers::default();

    let attacker = Uuid::from_u128(1);
    let defender = Uuid::from_u128(2);

    let mut match_ = TeamDeathmatch::new(vec![attacker, defender], 3, 60.0);

    let mut shield = 90.0_f32;
    let mut armor = 70.0_f32;
    let dmg = 25.0_f32;
    let dtype = DamageType::Electromagnetic;

    let mut shots = 0;
    loop {
        let r = apply_damage(dtype, dmg, shield, armor, &mult);
        shots += 1;
        shield = r.shield_remaining;
        armor = r.armor_remaining;
        if r.ship_destroyed {
            break;
        }
    }
    assert_eq!(shots, 7, "EM kills a Recon in 7 shots vs 11 for Kinetic");
    assert_eq!(armor, 0.0);
    assert_eq!(shield, 0.0);

    match_.on_player_death(defender, Some(attacker));
    match_.on_player_death(defender, Some(attacker));
    match_.on_player_death(defender, Some(attacker));
    assert!(match_.is_finished());

    let r = match_.results().unwrap();
    let a = r.players.iter().find(|p| p.player_id == attacker).unwrap();
    assert_eq!(a.kills, 3);
    assert_eq!(r.team_scores[&0], 3);
}

#[test]
fn overkill_through_armor_still_destroys() {
    let mult = DamageMultipliers::default();

    let attacker = Uuid::from_u128(1);
    let defender = Uuid::from_u128(2);

    let mut match_ = TeamDeathmatch::new(vec![attacker, defender], 1, 60.0);

    let shield = 0.0_f32;
    let armor = 1.0_f32;
    let dmg = 100.0_f32;
    let dtype = DamageType::Thermic;

    let r = apply_damage(dtype, dmg, shield, armor, &mult);
    assert!(r.ship_destroyed);
    assert_eq!(r.armor_remaining, 0.0);
    assert!((r.mitigated - 99.0).abs() < 1e-4);

    match_.on_player_death(defender, Some(attacker));
    assert!(match_.is_finished());

    let r = match_.results().unwrap();
    let a = r.players.iter().find(|p| p.player_id == attacker).unwrap();
    assert_eq!(a.kills, 1);
    assert_eq!(r.team_scores[&0], 1);
}
