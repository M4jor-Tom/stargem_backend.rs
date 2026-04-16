use crate::domain::GameMode;
use crate::game::GameInstanceManager;
use crate::AppError;
use std::collections::VecDeque;
use uuid::Uuid;

pub struct Matchmaker {
    queue: VecDeque<MatchmakingTicket>,
    min_players: usize,
    max_players: usize,
}

impl Matchmaker {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            min_players: 2,
            max_players: 16,
        }
    }

    pub fn enter_queue(&mut self, player_id: Uuid, preferred_mode: GameMode) {
        if !self.queue.iter().any(|t| t.player_id == player_id) {
            self.queue.push_back(MatchmakingTicket {
                player_id,
                preferred_mode,
                entered_at: chrono::Utc::now(),
            });
        }
    }

    pub fn leave_queue(&mut self, player_id: Uuid) {
        self.queue.retain(|t| t.player_id != player_id);
    }

    pub fn try_match(&mut self) -> Option<Vec<Uuid>> {
        if self.queue.len() < self.min_players {
            return None;
        }

        let mut matched_players = Vec::with_capacity(self.max_players);
        let mut mode_counts: std::collections::HashMap<GameMode, usize> =
            std::collections::HashMap::new();

        let mut remaining: VecDeque<MatchmakingTicket> = VecDeque::new();

        while let Some(ticket) = self.queue.pop_front() {
            if matched_players.len() >= self.max_players {
                remaining.push_back(ticket);
                continue;
            }

            let mode_count = mode_counts.entry(ticket.preferred_mode).or_insert(0);

            if *mode_count == 0 || matched_players.len() < self.min_players {
                matched_players.push(ticket.player_id);
                *mode_count += 1;
            } else {
                remaining.push_back(ticket);
            }
        }

        self.queue = remaining;

        if matched_players.len() >= self.min_players {
            Some(matched_players)
        } else {
            for player_id in matched_players {
                self.queue.push_back(MatchmakingTicket {
                    player_id,
                    preferred_mode: GameMode::TeamDeathmatch,
                    entered_at: chrono::Utc::now(),
                });
            }
            None
        }
    }

    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }

    pub fn is_in_queue(&self, player_id: Uuid) -> bool {
        self.queue.iter().any(|t| t.player_id == player_id)
    }

    pub fn clear_stale_entries(&mut self, max_wait_seconds: i64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(max_wait_seconds);
        self.queue.retain(|t| t.entered_at > cutoff);
    }
}

impl Default for Matchmaker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MatchmakingTicket {
    pub player_id: Uuid,
    pub preferred_mode: GameMode,
    pub entered_at: chrono::DateTime<chrono::Utc>,
}

pub struct QueueManager {
    pvp_matchmaker: Matchmaker,
    pve_matchmaker: Matchmaker,
    open_world_manager: OpenWorldManager,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            pvp_matchmaker: Matchmaker::new(),
            pve_matchmaker: Matchmaker::new(),
            open_world_manager: OpenWorldManager::new(),
        }
    }

    pub fn enter_queue(&mut self, player_id: Uuid, mode: GameMode) {
        match mode {
            GameMode::TeamDeathmatch | GameMode::FreeForAll => {
                self.pvp_matchmaker.enter_queue(player_id, mode);
            }
            GameMode::WavesSurvival | GameMode::OperationScenario => {
                self.pve_matchmaker.enter_queue(player_id, mode);
            }
            GameMode::OpenWorld => {
                self.open_world_manager.enter(player_id);
            }
        }
    }

    pub fn leave_queue(&mut self, player_id: Uuid) {
        self.pvp_matchmaker.leave_queue(player_id);
        self.pve_matchmaker.leave_queue(player_id);
        self.open_world_manager.leave(player_id);
    }

    pub fn try_match_pvp(&mut self) -> Option<Vec<Uuid>> {
        self.pvp_matchmaker.try_match()
    }

    pub fn try_match_pve(&mut self) -> Option<Vec<Uuid>> {
        self.pve_matchmaker.try_match()
    }

    pub fn is_in_queue(&self, player_id: Uuid) -> bool {
        self.pvp_matchmaker.is_in_queue(player_id)
            || self.pve_matchmaker.is_in_queue(player_id)
            || self.open_world_manager.is_in(player_id)
    }

    pub fn cleanup_stale(&mut self) {
        self.pvp_matchmaker.clear_stale_entries(300);
        self.pve_matchmaker.clear_stale_entries(300);
    }
}

impl Default for QueueManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OpenWorldManager {
    players: std::collections::HashSet<Uuid>,
    instances: Vec<Uuid>,
}

impl OpenWorldManager {
    pub fn new() -> Self {
        Self {
            players: std::collections::HashSet::new(),
            instances: Vec::new(),
        }
    }

    pub fn enter(&mut self, player_id: Uuid) {
        self.players.insert(player_id);
    }

    pub fn leave(&mut self, player_id: Uuid) {
        self.players.remove(&player_id);
    }

    pub fn is_in(&self, player_id: Uuid) -> bool {
        self.players.contains(&player_id)
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    pub fn assign_instance(&mut self) -> Uuid {
        if let Some(&instance_id) = self.instances.last() {
            return instance_id;
        }
        Uuid::new_v4()
    }
}

impl Default for OpenWorldManager {
    fn default() -> Self {
        Self::new()
    }
}
