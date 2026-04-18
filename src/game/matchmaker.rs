use crate::domain::GameMode;
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

        let mut matched_tickets: VecDeque<MatchmakingTicket> = VecDeque::new();
        let mut mode_counts: std::collections::HashMap<GameMode, usize> =
            std::collections::HashMap::new();

        let mut remaining: VecDeque<MatchmakingTicket> = VecDeque::new();

        while let Some(ticket) = self.queue.pop_front() {
            if matched_tickets.len() >= self.max_players {
                remaining.push_back(ticket);
                continue;
            }

            let mode_count = mode_counts.entry(ticket.preferred_mode).or_insert(0);

            if *mode_count == 0 || matched_tickets.len() < self.min_players {
                matched_tickets.push_back(ticket);
                *mode_count += 1;
            } else {
                remaining.push_back(ticket);
            }
        }

        self.queue = remaining;

        if matched_tickets.len() >= self.min_players {
            Some(matched_tickets.into_iter().map(|t| t.player_id).collect())
        } else {
            for ticket in matched_tickets {
                self.queue.push_back(ticket);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::GameMode;

    #[test]
    fn test_matchmaker_empty_queue() {
        let mut matchmaker = Matchmaker::new();
        assert!(matchmaker.try_match().is_none());
        assert_eq!(matchmaker.queue_size(), 0);
    }

    #[test]
    fn test_matchmaker_add_to_queue() {
        let mut matchmaker = Matchmaker::new();
        let player_id = Uuid::new_v4();

        matchmaker.enter_queue(player_id, GameMode::TeamDeathmatch);

        assert_eq!(matchmaker.queue_size(), 1);
        assert!(matchmaker.is_in_queue(player_id));
    }

    #[test]
    fn test_matchmaker_leave_queue() {
        let mut matchmaker = Matchmaker::new();
        let player_id = Uuid::new_v4();

        matchmaker.enter_queue(player_id, GameMode::TeamDeathmatch);
        matchmaker.leave_queue(player_id);

        assert_eq!(matchmaker.queue_size(), 0);
        assert!(!matchmaker.is_in_queue(player_id));
    }

    #[test]
    fn test_matchmaker_no_duplicate() {
        let mut matchmaker = Matchmaker::new();
        let player_id = Uuid::new_v4();

        matchmaker.enter_queue(player_id, GameMode::TeamDeathmatch);
        matchmaker.enter_queue(player_id, GameMode::TeamDeathmatch);

        assert_eq!(matchmaker.queue_size(), 1);
    }

    #[test]
    fn test_matchmaker_insufficient_players() {
        let mut matchmaker = Matchmaker::new();

        matchmaker.enter_queue(Uuid::new_v4(), GameMode::TeamDeathmatch);

        assert!(matchmaker.try_match().is_none());
        assert_eq!(matchmaker.queue_size(), 1);
    }

    #[test]
    fn test_matchmaker_successful_match() {
        let mut matchmaker = Matchmaker::new();

        for _ in 0..4 {
            matchmaker.enter_queue(Uuid::new_v4(), GameMode::TeamDeathmatch);
        }

        let matched = matchmaker.try_match();
        assert!(matched.is_some());
        assert!(matched.unwrap().len() >= 2);
        assert!(matchmaker.queue_size() <= 2);
    }

    #[test]
    fn test_matchmaker_max_players() {
        let mut matchmaker = Matchmaker::new();

        for _ in 0..20 {
            matchmaker.enter_queue(Uuid::new_v4(), GameMode::TeamDeathmatch);
        }

        let matched = matchmaker.try_match();
        assert!(matched.is_some());
        assert!(matched.unwrap().len() <= 16);
    }
}
