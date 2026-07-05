use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use tokio::sync::{broadcast, Mutex};
use tokio::time::interval;
use uuid::Uuid;

use stargem_backend::combat::damage::{apply_damage, DamageMultipliers};
use stargem_backend::combat::physics::PhysicsState;
use stargem_backend::combat::tick::{snapshot_to_proto, DamageEventRecord, PlayerState, TickSnapshot};
use stargem_backend::proto_gen::grpc::spectator::spectator_service_server::SpectatorServiceServer;
use stargem_backend::scenarios::{
    damage_type_str, ship_destruction_electromag, ship_destruction_kinetic,
    ship_destruction_overkill, Scenario, ScenarioAction,
};
use stargem_backend::spectator::{MatchRegistry, SpectatorHandler};

fn lookup(name: &str) -> Option<Scenario> {
    match name {
        "ship_destruction_kinetic" => Some(ship_destruction_kinetic()),
        "ship_destruction_electromag" => Some(ship_destruction_electromag()),
        "ship_destruction_overkill" => Some(ship_destruction_overkill()),
        _ => None,
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// Which scenario to run.
    #[arg(long)]
    scenario: String,

    /// gRPC listen address (spectator subscribers connect here).
    #[arg(long, default_value = "0.0.0.0:50051")]
    grpc_addr: String,

    /// Tick rate (Hz). Lower = slower playback; useful for watching by hand.
    #[arg(long, default_value_t = 2)]
    tick_rate: u64,

    /// Loop the scenario forever so a spectator can join at any time.
    #[arg(long)]
    loop_: bool,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let scn = lookup(&args.scenario)
        .ok_or_else(|| format!("unknown scenario: {}", args.scenario))?;

    let registry = Arc::new(Mutex::new(MatchRegistry::new()));
    let (tx, _) = broadcast::channel(256);
    let match_id = Uuid::new_v4();
    registry.lock().await.register(
        match_id,
        scn.spawns.iter().map(|s| s.player_id.to_string()).collect(),
        0,
        tx.clone(),
    );

    let addr: std::net::SocketAddr = args.grpc_addr.parse()?;
    let svc = SpectatorServiceServer::new(SpectatorHandler { registry: registry.clone() });
    tracing::info!("scenario-runner starting on {}, match_id={}", addr, match_id);

    let game = run_loop(scn, tx, args.tick_rate, args.loop_);
    let server = tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr);

    tokio::pin!(game);
    tokio::pin!(server);

    tokio::select! {
        r = &mut server => {
            r?;
        }
        _ = &mut game => {
            tracing::info!("scenario completed");
        }
    }
    Ok(())
}

async fn run_loop(
    scn: Scenario,
    tx: broadcast::Sender<stargem_backend::proto_gen::quic::combat::GameStateSnapshot>,
    tick_rate: u64,
    do_loop: bool,
) {
    let mult = DamageMultipliers::default();
    let mut interval = interval(Duration::from_secs_f64(1.0 / tick_rate as f64));

    loop {
        let mut ships: std::collections::HashMap<Uuid, PlayerState> = scn.spawns.iter().map(|s| {
            let mut phys = PhysicsState::new();
            phys.position = s.position;
            (s.player_id, PlayerState {
                id: s.player_id.to_string(),
                physics: phys,
                stats: s.stats.clone(),
                shield_hp: s.stats.current_shield,
                armor_hp: s.stats.current_armor,
                energy: s.stats.current_energy,
                heat_level: 0.0,
                input: Default::default(),
            })
        }).collect();

        let mut tick: u64 = 0;
        let mut script_idx = 0;
        let mut sorted: Vec<&_> = scn.script.iter().collect();
        sorted.sort_by_key(|s| s.at_tick);

        let mut ended = false;
        while !ended {
            interval.tick().await;
            tick += 1;

            let mut pending: Vec<DamageEventRecord> = Vec::new();

            while script_idx < sorted.len() && sorted[script_idx].at_tick <= tick {
                let step = sorted[script_idx];
                script_idx += 1;
                match &step.action {
                    ScenarioAction::Damage { src, tgt, dtype, raw } => {
                        if let Some(target) = ships.get_mut(tgt) {
                            let r = apply_damage(*dtype, *raw, target.shield_hp, target.armor_hp, &mult);
                            target.shield_hp = r.shield_remaining;
                            target.armor_hp = r.armor_remaining;
                            pending.push(DamageEventRecord {
                                source: src.to_string(),
                                target: tgt.to_string(),
                                damage_type: damage_type_str(*dtype).to_string(),
                                raw_amount: *raw,
                                mitigated_amount: r.mitigated,
                            });
                        }
                    }
                    ScenarioAction::Input { player, input } => {
                        if let Some(ship) = ships.get_mut(player) {
                            ship.input = input.clone();
                        }
                    }
                    ScenarioAction::EndMatch => { ended = true; }
                }
            }

            let snapshot = TickSnapshot {
                tick_number: tick,
                players: ships.values().cloned().collect(),
                damage_events: pending,
            };
            let _ = tx.send(snapshot_to_proto(&snapshot));
        }

        if !do_loop { break; }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}