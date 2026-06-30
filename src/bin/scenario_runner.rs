use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use tokio::sync::{broadcast, Mutex};
use tokio::time::interval;
use uuid::Uuid;

use stargem_backend::combat::damage::{apply_damage, DamageMultipliers};
use stargem_backend::combat::physics::PhysicsState;
use stargem_backend::combat::tick::{snapshot_to_proto, PlayerState, TickSnapshot};
use stargem_backend::proto_gen::grpc::spectator::spectator_service_server::SpectatorServiceServer;
use stargem_backend::scenarios::{
    damage_type_str, ship_destruction_electromag, ship_destruction_kinetic,
    ship_destruction_overkill, Scenario, ScenarioAction,
};
use stargem_backend::spectator::{MatchRegistry, SpectatorHandler};

fn init_scenarios() -> HashMap<&'static str, Scenario> {
    let mut m = HashMap::new();
    m.insert("kinetic", ship_destruction_kinetic());
    m.insert("electromag", ship_destruction_electromag());
    m.insert("overkill", ship_destruction_overkill());
    m
}

#[derive(Parser)]
#[command(name = "scenario-runner")]
struct Args {
    #[arg(long, default_value = "kinetic")]
    scenario: String,
    #[arg(long, default_value = "0.0.0.0:50051")]
    grpc_addr: String,
    #[arg(long, default_value_t = 60)]
    tick_rate: u64,
    #[arg(long)]
    loop_: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let scenarios = init_scenarios();
    let scn = scenarios.get(args.scenario.as_str())
        .expect("unknown scenario (kinetic|electromag|overkill)");

    let match_id = Uuid::from_u128(0xCAFE);
    let (tx, _) = broadcast::channel(64);

    let registry = Arc::new(Mutex::new(MatchRegistry::new()));
    registry.lock().await.register(
        match_id,
        scn.spawns.iter().map(|s| s.player_id.to_string()).collect(),
        0,
        tx.clone(),
    );

    let handler = SpectatorHandler { registry: registry.clone() };
    let svc = SpectatorServiceServer::new(handler);
    let grpc_addr = args.grpc_addr.clone();

    tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(svc)
            .serve(grpc_addr.parse().unwrap())
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    tracing::info!("Scenario '{}' ready on {}", scn.name, args.grpc_addr);

    let tick_dt = Duration::from_secs_f64(1.0 / args.tick_rate as f64);
    let mut current_tick: u64 = 0;
    let mut script_idx: usize = 0;
    let mut shield_hp: HashMap<Uuid, f32> = HashMap::new();
    let mut armor_hp: HashMap<Uuid, f32> = HashMap::new();
    let mut player_states: HashMap<Uuid, PlayerState> = HashMap::new();

    for spawn in &scn.spawns {
        shield_hp.insert(spawn.player_id, spawn.stats.current_shield);
        armor_hp.insert(spawn.player_id, spawn.stats.current_armor);
        let mut p = PhysicsState::new();
        p.position = spawn.position;
        player_states.insert(spawn.player_id, PlayerState {
            id: spawn.player_id.to_string(),
            physics: p,
            stats: spawn.stats.clone(),
            shield_hp: spawn.stats.current_shield,
            armor_hp: spawn.stats.current_armor,
            energy: spawn.stats.current_energy,
            heat_level: 0.0,
            input: Default::default(),
        });
    }

    loop {
        interval(tick_dt).tick().await;
        current_tick += 1;

        while script_idx < scn.script.len() && scn.script[script_idx].at_tick <= current_tick {
            let step = &scn.script[script_idx];
            match &step.action {
                ScenarioAction::Damage { src, tgt, dtype, raw } => {
                    let cur_shield = *shield_hp.get(tgt).unwrap_or(&0.0);
                    let cur_armor = *armor_hp.get(tgt).unwrap_or(&0.0);
                    let mult = DamageMultipliers::default();
                    let result = apply_damage(*dtype, *raw, cur_shield, cur_armor, &mult);
                    shield_hp.insert(*tgt, result.shield_remaining);
                    armor_hp.insert(*tgt, result.armor_remaining);
                    if let Some(ps) = player_states.get_mut(tgt) {
                        ps.shield_hp = result.shield_remaining;
                        ps.armor_hp = result.armor_remaining;
                    }
                    tracing::info!(
                        "tick={} damage src={} tgt={} type={} shield={:.1}->{:.1} armor={:.1}->{:.1}",
                        current_tick, src, tgt, damage_type_str(*dtype),
                        cur_shield, result.shield_remaining, cur_armor, result.armor_remaining,
                    );
                }
                ScenarioAction::Input { .. } => {}
                ScenarioAction::EndMatch => {
                    tracing::info!("Scenario complete at tick {}", current_tick);
                    if args.loop_ {
                        tracing::info!("Restarting scenario...");
                        current_tick = 0;
                        script_idx = 0;
                        shield_hp.clear();
                        armor_hp.clear();
                        player_states.clear();
                        for spawn in &scn.spawns {
                            shield_hp.insert(spawn.player_id, spawn.stats.current_shield);
                            armor_hp.insert(spawn.player_id, spawn.stats.current_armor);
                            let mut p = PhysicsState::new();
                            p.position = spawn.position;
                            player_states.insert(spawn.player_id, PlayerState {
                                id: spawn.player_id.to_string(),
                                physics: p,
                                stats: spawn.stats.clone(),
                                shield_hp: spawn.stats.current_shield,
                                armor_hp: spawn.stats.current_armor,
                                energy: spawn.stats.current_energy,
                                heat_level: 0.0,
                                input: Default::default(),
                            });
                        }
                    }
                }
            }
            script_idx += 1;
        }

        let snapshot = TickSnapshot {
            tick_number: current_tick,
            players: player_states.values().cloned().collect(),
            damage_events: vec![],
        };
        let _ = tx.send(snapshot_to_proto(&snapshot));
    }
}