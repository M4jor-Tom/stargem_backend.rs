#![allow(dead_code)]

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub type TeamId = u8;

#[derive(Debug, Clone)]
pub struct PlayerInMatch {
    pub player_id: Uuid,
    pub team_id: TeamId,
    pub kills: u32,
    pub deaths: u32,
    pub damage_dealt: f32,
    pub damage_taken: f32,
}

#[derive(Debug, Clone)]
pub struct MatchResults {
    pub match_id: Uuid,
    pub winning_team: Option<TeamId>,
    pub team_scores: HashMap<TeamId, u32>,
    pub players: Vec<PlayerInMatch>,
    pub duration_secs: f64,
    pub reason: String,
}

#[async_trait]
pub trait GameMode: Send + Sync {
    fn on_player_death(&mut self, player_id: Uuid, killer_id: Option<Uuid>);
    fn on_tick(&mut self, dt_secs: f64);
    fn is_finished(&self) -> bool;
    fn results(&self) -> Option<MatchResults>;
    fn teams(&self) -> &[Vec<Uuid>];
}

pub struct TeamDeathmatch {
    match_id: Uuid,
    teams: Vec<Vec<Uuid>>,
    players: HashMap<Uuid, PlayerInMatch>,
    team_scores: HashMap<TeamId, u32>,
    score_limit: u32,
    time_limit_secs: f64,
    elapsed_secs: f64,
    finished: bool,
    results: Option<MatchResults>,
}

impl TeamDeathmatch {
    pub fn new(players: Vec<Uuid>, score_limit: u32, time_limit_secs: f64) -> Self {
        let match_id = Uuid::new_v4();
        let mid = players.len() / 2;
        let mut team_scores = HashMap::new();

        let teams = vec![players[..mid].to_vec(), players[mid..].to_vec()];

        let mut player_map = HashMap::new();
        for (team_idx, team) in teams.iter().enumerate() {
            let team_id = team_idx as TeamId;
            team_scores.insert(team_id, 0);
            for &pid in team {
                player_map.insert(
                    pid,
                    PlayerInMatch {
                        player_id: pid,
                        team_id,
                        kills: 0,
                        deaths: 0,
                        damage_dealt: 0.0,
                        damage_taken: 0.0,
                    },
                );
            }
        }

        Self {
            match_id,
            teams,
            players: player_map,
            team_scores,
            score_limit,
            time_limit_secs,
            elapsed_secs: 0.0,
            finished: false,
            results: None,
        }
    }
}

impl GameMode for TeamDeathmatch {
    fn on_player_death(&mut self, player_id: Uuid, killer_id: Option<Uuid>) {
        if let Some(player) = self.players.get_mut(&player_id) {
            player.deaths += 1;
            player.damage_taken += 100.0; // TODO: replace with actual damage from combat system
        }
        if let Some(killer) = killer_id {
            if let Some(killer_player) = self.players.get_mut(&killer) {
                killer_player.kills += 1;
                killer_player.damage_dealt += 100.0; // TODO: replace with actual damage from combat system
                let team_id = killer_player.team_id;
                if let Some(score) = self.team_scores.get_mut(&team_id) {
                    *score += 1;
                    if *score >= self.score_limit {
                        self.finished = true;
                        self.results = Some(MatchResults {
                            match_id: self.match_id,
                            winning_team: Some(team_id),
                            team_scores: self.team_scores.clone(),
                            players: self.players.values().cloned().collect(),
                            duration_secs: self.elapsed_secs,
                            reason: "score_limit".into(),
                        });
                    }
                }
            }
        }
    }

    fn on_tick(&mut self, dt_secs: f64) {
        if self.finished {
            return;
        }
        self.elapsed_secs += dt_secs;
        if self.elapsed_secs >= self.time_limit_secs {
            self.finished = true;
            let winning_team = self
                .team_scores
                .iter()
                .max_by_key(|(_, &score)| score)
                .map(|(&team, _)| team);

            self.results = Some(MatchResults {
                match_id: self.match_id,
                winning_team,
                team_scores: self.team_scores.clone(),
                players: self.players.values().cloned().collect(),
                duration_secs: self.elapsed_secs,
                reason: "time_limit".into(),
            });
        }
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn results(&self) -> Option<MatchResults> {
        self.results.clone()
    }

    fn teams(&self) -> &[Vec<Uuid>] {
        &self.teams
    }
}

pub struct MatchManager {
    queue: Vec<Uuid>,
    active_match: Option<Arc<Mutex<TeamDeathmatch>>>,
    min_players: usize,
    max_players: usize,
}

impl MatchManager {
    pub fn new(min_players: usize, max_players: usize) -> Self {
        Self {
            queue: Vec::new(),
            active_match: None,
            min_players,
            max_players,
        }
    }

    pub fn enqueue(&mut self, player_id: Uuid) -> usize {
        if !self.queue.contains(&player_id) {
            self.queue.push(player_id);
        }
        self.queue.len()
    }

    pub fn dequeue(&mut self, player_id: &Uuid) -> bool {
        let len_before = self.queue.len();
        self.queue.retain(|p| p != player_id);
        self.queue.len() < len_before
    }

    pub fn queue_position(&self, player_id: &Uuid) -> Option<usize> {
        self.queue.iter().position(|p| p == player_id)
    }

    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }

    pub fn try_start_match(&mut self) -> Option<Arc<Mutex<TeamDeathmatch>>> {
        if self.queue.len() >= self.min_players && self.active_match.is_none() {
            let players: Vec<Uuid> = self
                .queue
                .drain(..self.max_players.min(self.queue.len()))
                .collect();
            let match_instance = Arc::new(Mutex::new(TeamDeathmatch::new(players, 50, 600.0)));
            self.active_match = Some(match_instance.clone());
            Some(match_instance)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_deathmatch_splits_even_players() {
        let p: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();
        let tdm = TeamDeathmatch::new(p.clone(), 50, 600.0);
        assert_eq!(tdm.teams.len(), 2);
        assert_eq!(tdm.teams[0].len(), 2);
        assert_eq!(tdm.teams[1].len(), 2);
    }

    #[test]
    fn test_team_deathmatch_splits_odd_players() {
        let p: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
        let tdm = TeamDeathmatch::new(p.clone(), 50, 600.0);
        assert_eq!(tdm.teams[0].len(), 2);
        assert_eq!(tdm.teams[1].len(), 3);
    }

    #[test]
    fn test_death_awards_kill_and_increments_score() {
        let p: Vec<Uuid> = (0..2).map(|_| Uuid::new_v4()).collect();
        let mut tdm = TeamDeathmatch::new(p.clone(), 50, 600.0);
        tdm.on_player_death(p[1], Some(p[0]));
        assert_eq!(tdm.players.get(&p[0]).unwrap().kills, 1);
        assert_eq!(tdm.players.get(&p[1]).unwrap().deaths, 1);
        assert_eq!(tdm.team_scores[&0], 1);
        assert!(!tdm.is_finished());
    }

    #[test]
    fn test_score_limit_triggers_win() {
        let p: Vec<Uuid> = (0..2).map(|_| Uuid::new_v4()).collect();
        let mut tdm = TeamDeathmatch::new(p.clone(), 2, 600.0);
        tdm.on_player_death(p[1], Some(p[0]));
        assert!(!tdm.is_finished());
        tdm.on_player_death(p[1], Some(p[0]));
        assert!(tdm.is_finished());
        let r = tdm.results().unwrap();
        assert_eq!(r.winning_team, Some(0));
        assert_eq!(r.reason, "score_limit");
    }

    #[test]
    fn test_time_limit_triggers_win() {
        let p: Vec<Uuid> = (0..2).map(|_| Uuid::new_v4()).collect();
        let mut tdm = TeamDeathmatch::new(p.clone(), 50, 10.0);
        tdm.on_tick(15.0);
        assert!(tdm.is_finished());
        assert_eq!(tdm.results().unwrap().reason, "time_limit");
    }

    #[test]
    fn test_time_limit_does_not_override_score_limit() {
        let p: Vec<Uuid> = (0..2).map(|_| Uuid::new_v4()).collect();
        let mut tdm = TeamDeathmatch::new(p.clone(), 1, 10.0);
        tdm.on_player_death(p[1], Some(p[0]));
        tdm.on_tick(15.0);
        assert_eq!(tdm.results().unwrap().reason, "score_limit");
    }

    #[test]
    fn test_death_without_killer_no_score() {
        let p: Vec<Uuid> = (0..2).map(|_| Uuid::new_v4()).collect();
        let mut tdm = TeamDeathmatch::new(p.clone(), 50, 600.0);
        tdm.on_player_death(p[0], None);
        assert_eq!(tdm.players.get(&p[0]).unwrap().deaths, 1);
        assert!(tdm.team_scores.values().all(|&s| s == 0));
    }

    #[test]
    fn test_match_manager_enqueue_and_position() {
        let mut mgr = MatchManager::new(4, 16);
        let pid = Uuid::new_v4();
        assert_eq!(mgr.enqueue(pid), 1);
        assert_eq!(mgr.queue_position(&pid), Some(0));
        assert_eq!(mgr.queue_size(), 1);
    }

    #[test]
    fn test_match_manager_dequeue() {
        let mut mgr = MatchManager::new(4, 16);
        let pid = Uuid::new_v4();
        mgr.enqueue(pid);
        assert!(mgr.dequeue(&pid));
        assert!(!mgr.dequeue(&pid));
        assert_eq!(mgr.queue_size(), 0);
    }

    #[test]
    fn test_match_manager_no_double_enqueue() {
        let mut mgr = MatchManager::new(4, 16);
        let pid = Uuid::new_v4();
        mgr.enqueue(pid);
        mgr.enqueue(pid);
        assert_eq!(mgr.queue_size(), 1);
    }

    #[test]
    fn test_try_start_match_not_enough_players() {
        let mut mgr = MatchManager::new(4, 16);
        for _ in 0..3 {
            mgr.enqueue(Uuid::new_v4());
        }
        assert!(mgr.try_start_match().is_none());
    }

    #[test]
    fn test_try_start_match_drains_queue_and_blocks_reentry() {
        let mut mgr = MatchManager::new(2, 16);
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        mgr.enqueue(p1);
        mgr.enqueue(p2);
        assert!(mgr.try_start_match().is_some());
        assert_eq!(mgr.queue_size(), 0);
        assert!(mgr.try_start_match().is_none());
    }

    #[test]
    fn test_active_match_blocks_new_match() {
        let mut mgr = MatchManager::new(4, 8);
        for _ in 0..8 {
            mgr.enqueue(Uuid::new_v4());
        }
        assert!(mgr.try_start_match().is_some());
        for _ in 0..4 {
            mgr.enqueue(Uuid::new_v4());
        }
        assert!(mgr.try_start_match().is_none());
    }

    #[test]
    fn test_empty_queue_try_start_match_returns_none() {
        let mut mgr = MatchManager::new(4, 16);
        assert!(mgr.try_start_match().is_none());
    }

    #[test]
    fn test_on_tick_after_finished_is_noop() {
        let p: Vec<Uuid> = (0..2).map(|_| Uuid::new_v4()).collect();
        let mut tdm = TeamDeathmatch::new(p.clone(), 1, 600.0);
        tdm.on_player_death(p[1], Some(p[0]));
        assert!(tdm.is_finished());
        let before = tdm.elapsed_secs;
        tdm.on_tick(10.0);
        assert_eq!(tdm.elapsed_secs, before);
    }

    #[test]
    fn test_dequeue_removes_correct_player_and_shifts_positions() {
        let mut mgr = MatchManager::new(4, 16);
        let players: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
        for &p in &players {
            mgr.enqueue(p);
        }
        assert_eq!(mgr.queue_size(), 5);
        assert!(mgr.dequeue(&players[2]));
        assert_eq!(mgr.queue_size(), 4);
        assert_eq!(mgr.queue_position(&players[0]), Some(0));
        assert_eq!(mgr.queue_position(&players[1]), Some(1));
        assert_eq!(mgr.queue_position(&players[2]), None);
        assert_eq!(mgr.queue_position(&players[3]), Some(2));
        assert_eq!(mgr.queue_position(&players[4]), Some(3));
    }

    #[test]
    fn test_match_duration_tracking() {
        let p: Vec<Uuid> = (0..2).map(|_| Uuid::new_v4()).collect();
        let mut tdm = TeamDeathmatch::new(p.clone(), 50, 600.0);
        for _ in 0..10 {
            tdm.on_tick(1.0);
        }
        assert_eq!(tdm.elapsed_secs, 10.0);
    }

    #[test]
    fn test_damage_stats_accumulate_on_death() {
        let players = vec![Uuid::from_u128(1), Uuid::from_u128(2)];
        let mut match_ = TeamDeathmatch::new(players.clone(), 50, 600.0);

        match_.on_player_death(players[1], Some(players[0]));

        let killer = match_.players.get(&players[0]).unwrap();
        let victim = match_.players.get(&players[1]).unwrap();

        assert!(
            killer.damage_dealt > 0.0,
            "killer should have damage_dealt > 0, got {}",
            killer.damage_dealt
        );
        assert!(
            victim.damage_taken > 0.0,
            "victim should have damage_taken > 0, got {}",
            victim.damage_taken
        );
    }
}
