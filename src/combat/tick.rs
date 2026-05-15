use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

use crate::combat::damage::{apply_damage, load_damage_multipliers, DamageMultipliers, DamageType};
use crate::combat::physics::{PhysicsState, ShipInput};
use crate::ship::stats::PlayerShipStats;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: String,
    pub physics: PhysicsState,
    pub stats: PlayerShipStats,
    pub shield_hp: f32,
    pub armor_hp: f32,
    pub energy: f32,
    pub heat_level: f32,
    pub input: ShipInput,
}

#[derive(Debug, Clone)]
pub struct TickSnapshot {
    pub tick_number: u64,
    pub players: Vec<PlayerState>,
    pub damage_events: Vec<DamageEventRecord>,
}

#[derive(Debug, Clone)]
pub struct DamageEventRecord {
    pub source: String,
    pub target: String,
    pub damage_type: String,
    pub raw_amount: f32,
    pub mitigated_amount: f32,
}

pub struct CombatTickLoop {
    tick_rate: u64,
    tick_number: u64,
    players: HashMap<String, PlayerState>,
    damage_multipliers: DamageMultipliers,
    snapshot_tx: mpsc::Sender<TickSnapshot>,
    input_rx: mpsc::Receiver<(String, ShipInput)>,
}

impl CombatTickLoop {
    pub fn new(
        tick_rate: u64,
        snapshot_tx: mpsc::Sender<TickSnapshot>,
        input_rx: mpsc::Receiver<(String, ShipInput)>,
    ) -> Self {
        let damage_multipliers = load_damage_multipliers("config/damage_multipliers.toml");
        Self {
            tick_rate,
            tick_number: 0,
            players: HashMap::new(),
            damage_multipliers,
            snapshot_tx,
            input_rx,
        }
    }

    pub fn add_player(&mut self, id: String, stats: PlayerShipStats) {
        let state = PlayerState {
            id: id.clone(),
            physics: PhysicsState::new(),
            stats,
            shield_hp: 0.0,
            armor_hp: 0.0,
            energy: 0.0,
            heat_level: 0.0,
            input: ShipInput {
                throttle: 0.0,
                yaw: 0.0,
                pitch: 0.0,
                roll: 0.0,
            },
        };
        self.players.insert(id, state);
    }

    pub async fn run(&mut self) {
        let dt = 1.0 / self.tick_rate as f32;
        let mut interval = time::interval(Duration::from_secs_f64(dt as f64));

        loop {
            interval.tick().await;
            self.tick_number += 1;

            while let Ok((id, input)) = self.input_rx.try_recv() {
                if let Some(player) = self.players.get_mut(&id) {
                    player.input = input;
                }
            }

            if self.players.is_empty() {
                continue;
            }

            for (_, player) in self.players.iter_mut() {
                player.physics.update(&player.input, &player.stats, dt);
            }

            let snapshot = TickSnapshot {
                tick_number: self.tick_number,
                players: self.players.values().cloned().collect(),
                damage_events: Vec::new(),
            };

            if self.snapshot_tx.try_send(snapshot).is_err() {
                tracing::warn!("Snapshot channel full, dropping tick {}", self.tick_number);
            }
        }
    }
}
