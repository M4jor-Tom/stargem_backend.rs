mod common;

use common::scenarios::{
    run_scenario_sync,
    ship_destruction_electromag, ship_destruction_kinetic, ship_destruction_overkill,
};
use uuid::Uuid;

#[test]
fn gunship_cannon_destroys_recon_in_team_deathmatch() {
    let scn = ship_destruction_kinetic();
    let out = run_scenario_sync(&scn);
    let defender = Uuid::from_u128(2);
    let final_ = &out.final_ships[&defender];
    assert!(final_.destroyed, "defender should be destroyed by shot 11");
    assert_eq!(final_.armor, 0.0);
    assert_eq!(out.damage_log.len(), 11);
}

#[test]
fn electromag_pierces_shield_quickly() {
    let scn = ship_destruction_electromag();
    let out = run_scenario_sync(&scn);
    let defender = Uuid::from_u128(2);
    let final_ = &out.final_ships[&defender];
    assert!(final_.destroyed, "EM kills recon in <=7 shots");
    assert_eq!(final_.armor, 0.0);
    assert_eq!(final_.shield, 0.0);
}

#[test]
fn overkill_through_armor_still_destroys() {
    let scn = ship_destruction_overkill();
    let out = run_scenario_sync(&scn);
    let defender = Uuid::from_u128(2);
    let final_ = &out.final_ships[&defender];
    assert!(final_.destroyed);
    assert_eq!(final_.armor, 0.0);
    assert!((out.damage_log[0].mitigated_amount - 99.0).abs() < 1e-4);
}