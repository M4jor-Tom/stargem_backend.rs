use std::collections::HashMap;
use uuid::Uuid;

use stargem_backend::combat::damage::{apply_damage, DamageMultipliers, DamageType};
use stargem_backend::combat::physics::ShipInput;
use stargem_backend::combat::tick::DamageEventRecord;
use stargem_backend::ship::stats::PlayerShipStats;

pub struct Scenario {
    pub name: &'static str,
    pub spawns: Vec<Spawn>,
    pub script: Vec<ScriptStep>,
}

pub struct Spawn {
    pub player_id: Uuid,
    pub position: [f32; 3],
    pub stats: PlayerShipStats,
}

pub struct ScriptStep {
    pub at_tick: u64,
    pub action: ScenarioAction,
}

pub enum ScenarioAction {
    Input { player: Uuid, input: ShipInput },
    Damage { src: Uuid, tgt: Uuid, dtype: DamageType, raw: f32 },
    EndMatch,
}

pub struct ScenarioOutcome {
    pub damage_log: Vec<DamageEventRecord>,
    pub final_ships: HashMap<Uuid, ShipFinal>,
}

pub struct ShipFinal {
    pub destroyed: bool,
    pub shield: f32,
    pub armor: f32,
}

pub fn damage_type_str(d: DamageType) -> &'static str {
    match d {
        DamageType::Kinetic => "kinetic",
        DamageType::Electromagnetic => "electromagnetic",
        DamageType::Thermic => "thermic",
    }
}

fn recon_stats() -> PlayerShipStats {
    PlayerShipStats {
        max_shield: 90.0, max_armor: 70.0, max_energy: 100.0,
        speed: 50.0, agility: 10.0,
        current_shield: 90.0, current_armor: 70.0, current_energy: 100.0,
    }
}

fn gunship_stats() -> PlayerShipStats {
    PlayerShipStats {
        max_shield: 90.0, max_armor: 70.0, max_energy: 100.0,
        speed: 50.0, agility: 10.0,
        current_shield: 90.0, current_armor: 70.0, current_energy: 100.0,
    }
}

pub fn ship_destruction_kinetic() -> Scenario {
    let attacker = Uuid::from_u128(1);
    let defender = Uuid::from_u128(2);
    let mut script = Vec::new();
    for shot in 1..=11 {
        script.push(ScriptStep {
            at_tick: shot,
            action: ScenarioAction::Damage {
                src: attacker, tgt: defender,
                dtype: DamageType::Kinetic, raw: 25.0,
            },
        });
    }
    script.push(ScriptStep { at_tick: 12, action: ScenarioAction::EndMatch });
    Scenario {
        name: "gunship_cannon_destroys_recon",
        spawns: vec![
            Spawn { player_id: attacker, position: [0.0, 0.0, 0.0], stats: gunship_stats() },
            Spawn { player_id: defender, position: [50.0, 0.0, 0.0], stats: recon_stats() },
        ],
        script,
    }
}

pub fn ship_destruction_electromag() -> Scenario {
    let attacker = Uuid::from_u128(1);
    let defender = Uuid::from_u128(2);
    let mut script = Vec::new();
    for shot in 1..=7 {
        script.push(ScriptStep {
            at_tick: shot,
            action: ScenarioAction::Damage {
                src: attacker, tgt: defender,
                dtype: DamageType::Electromagnetic, raw: 25.0,
            },
        });
    }
    script.push(ScriptStep { at_tick: 8, action: ScenarioAction::EndMatch });
    Scenario {
        name: "electromag_pierces_shield_quickly",
        spawns: vec![
            Spawn { player_id: attacker, position: [0.0, 0.0, 0.0], stats: gunship_stats() },
            Spawn { player_id: defender, position: [40.0, 0.0, 0.0], stats: recon_stats() },
        ],
        script,
    }
}

pub fn ship_destruction_overkill() -> Scenario {
    let attacker = Uuid::from_u128(1);
    let defender = Uuid::from_u128(2);
    let mut def_stats = recon_stats();
    def_stats.current_shield = 0.0;
    def_stats.current_armor = 1.0;
    Scenario {
        name: "overkill_through_armor_still_destroys",
        spawns: vec![
            Spawn { player_id: attacker, position: [0.0, 0.0, 0.0], stats: gunship_stats() },
            Spawn { player_id: defender, position: [30.0, 0.0, 0.0], stats: def_stats },
        ],
        script: vec![
            ScriptStep { at_tick: 1, action: ScenarioAction::Damage {
                src: attacker, tgt: defender,
                dtype: DamageType::Thermic, raw: 100.0,
            }},
            ScriptStep { at_tick: 2, action: ScenarioAction::EndMatch },
        ],
    }
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