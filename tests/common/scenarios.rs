use std::collections::HashMap;
use uuid::Uuid;

use stargem_backend::combat::damage::{apply_damage, DamageMultipliers};
use stargem_backend::combat::tick::DamageEventRecord;

pub use stargem_backend::scenarios::{
    damage_type_str, ship_destruction_electromag, ship_destruction_kinetic,
    ship_destruction_overkill, Scenario, ScenarioAction,
};

pub struct ScenarioOutcome {
    pub damage_log: Vec<DamageEventRecord>,
    pub final_ships: HashMap<Uuid, ShipFinal>,
}

pub struct ShipFinal {
    pub destroyed: bool,
    pub shield: f32,
    pub armor: f32,
}

pub fn run_scenario_sync(scn: &Scenario) -> ScenarioOutcome {
    let mult = DamageMultipliers::default();

    let mut shield_hp: HashMap<Uuid, f32> = HashMap::new();
    let mut armor_hp: HashMap<Uuid, f32> = HashMap::new();
    for spawn in &scn.spawns {
        shield_hp.insert(spawn.player_id, spawn.stats.current_shield);
        armor_hp.insert(spawn.player_id, spawn.stats.current_armor);
    }

    let mut damage_log: Vec<DamageEventRecord> = Vec::new();

    for step in &scn.script {
        match &step.action {
            ScenarioAction::Damage { src, tgt, dtype, raw } => {
                let cur_shield = *shield_hp.get(tgt).unwrap_or(&0.0);
                let cur_armor = *armor_hp.get(tgt).unwrap_or(&0.0);

                let result = apply_damage(*dtype, *raw, cur_shield, cur_armor, &mult);

                shield_hp.insert(*tgt, result.shield_remaining);
                armor_hp.insert(*tgt, result.armor_remaining);

                damage_log.push(DamageEventRecord {
                    source: src.to_string(),
                    target: tgt.to_string(),
                    damage_type: damage_type_str(*dtype).to_string(),
                    raw_amount: *raw,
                    mitigated_amount: result.mitigated,
                });
            }
            ScenarioAction::Input { .. } => {}
            ScenarioAction::EndMatch => break,
        }
    }

    let mut final_ships: HashMap<Uuid, ShipFinal> = HashMap::new();
    for spawn in &scn.spawns {
        let s = *shield_hp.get(&spawn.player_id).unwrap_or(&0.0);
        let a = *armor_hp.get(&spawn.player_id).unwrap_or(&0.0);
        final_ships.insert(spawn.player_id, ShipFinal {
            destroyed: a <= 0.0,
            shield: s,
            armor: a,
        });
    }

    ScenarioOutcome { damage_log, final_ships }
}