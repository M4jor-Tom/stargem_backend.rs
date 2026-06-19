use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};

use crate::combat::damage::{load_damage_multipliers, DamageMultipliers};
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
        let shield_hp = stats.current_shield;
        let armor_hp = stats.current_armor;
        let energy = stats.current_energy;
        let state = PlayerState {
            id: id.clone(),
            physics: PhysicsState::new(),
            stats,
            shield_hp,
            armor_hp,
            energy,
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn test_combat_tick_loop_new_defaults() {
        let (tx, _) = mpsc::channel(256);
        let (_, rx) = mpsc::channel(1024);
        let mut loop_ = CombatTickLoop::new(60, tx, rx);
        assert_eq!(loop_.tick_rate, 60);
        assert_eq!(loop_.tick_number, 0);
        assert!(loop_.players.is_empty());
    }

    #[test]
    fn test_combat_tick_loop_loads_default_damage_multipliers() {
        let (tx, _) = mpsc::channel(256);
        let (_, rx) = mpsc::channel(1024);
        let mut loop_ = CombatTickLoop::new(60, tx, rx);
        // When config file is absent, defaults are used
        assert_eq!(loop_.damage_multipliers.electromagnetic.shield, 1.5);
        assert_eq!(loop_.damage_multipliers.electromagnetic.armor, 0.5);
        assert_eq!(loop_.damage_multipliers.kinetic.shield, 0.5);
        assert_eq!(loop_.damage_multipliers.kinetic.armor, 1.5);
        assert_eq!(loop_.damage_multipliers.thermic.shield, 1.0);
        assert_eq!(loop_.damage_multipliers.thermic.armor, 1.0);
    }

    #[test]
    fn test_add_player_initializes_hp_from_stats() {
        let (snapshot_tx, _) = mpsc::channel(256);
        let (_, input_rx) = mpsc::channel(1024);
        let mut loop_ = CombatTickLoop::new(60, snapshot_tx, input_rx);

        let stats = PlayerShipStats {
            max_shield: 150.0,
            max_armor: 300.0,
            max_energy: 75.0,
            speed: 50.0,
            agility: 10.0,
            current_shield: 150.0,
            current_armor: 300.0,
            current_energy: 75.0,
        };

        loop_.add_player("p1".into(), stats.clone());

        let p = &loop_.players["p1"];
        assert!(
            (p.shield_hp - stats.current_shield).abs() < f32::EPSILON,
            "shield_hp should match stats.current_shield, got {} expected {}",
            p.shield_hp, stats.current_shield
        );
        assert!(
            (p.armor_hp - stats.current_armor).abs() < f32::EPSILON,
            "armor_hp should match stats.current_armor, got {} expected {}",
            p.armor_hp, stats.current_armor
        );
        assert!(
            (p.energy - stats.current_energy).abs() < f32::EPSILON,
            "energy should match stats.current_energy, got {} expected {}",
            p.energy, stats.current_energy
        );
    }

    #[test]
    fn test_add_player_inserts_player_state() {
        let (tx, _) = mpsc::channel(256);
        let (_, rx) = mpsc::channel(1024);
        let mut loop_ = CombatTickLoop::new(60, tx, rx);

        let stats = PlayerShipStats {
            max_shield: 100.0, max_armor: 100.0, max_energy: 100.0,
            speed: 50.0, agility: 10.0,
            current_shield: 100.0, current_armor: 100.0, current_energy: 100.0,
        };
        loop_.add_player("p1".into(), stats);
        assert_eq!(loop_.players.len(), 1);
        let p = &loop_.players["p1"];
        assert_eq!(p.id, "p1");
        assert_eq!(p.shield_hp, 100.0);
        assert_eq!(p.armor_hp, 100.0);
        assert_eq!(p.energy, 100.0);
        assert_eq!(p.heat_level, 0.0);
        assert_eq!(p.input.throttle, 0.0);
    }
}
