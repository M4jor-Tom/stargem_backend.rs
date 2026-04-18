use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameMode {
    TeamDeathmatch,
    FreeForAll,
    WavesSurvival,
    OperationScenario,
    OpenWorld,
}

impl GameMode {
    pub fn is_pvp(&self) -> bool {
        matches!(self, GameMode::TeamDeathmatch | GameMode::FreeForAll)
    }

    pub fn is_pve(&self) -> bool {
        matches!(self, GameMode::WavesSurvival | GameMode::OperationScenario)
    }

    pub fn is_open_world(&self) -> bool {
        matches!(self, GameMode::OpenWorld)
    }

    pub fn allows_respawn(&self) -> bool {
        matches!(
            self,
            GameMode::TeamDeathmatch | GameMode::FreeForAll | GameMode::WavesSurvival
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Lobby,
    Starting,
    InProgress,
    Paused,
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInstance {
    pub id: Uuid,
    pub mode: GameMode,
    pub state: GameState,
    pub name: String,
    pub max_players: usize,
    pub player_ids: Vec<Uuid>,
    pub team_scores: Option<Vec<i32>>,
    pub wave_number: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

impl GameInstance {
    pub fn new(name: String, mode: GameMode, max_players: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            mode,
            state: GameState::Lobby,
            name,
            max_players,
            player_ids: Vec::new(),
            team_scores: None,
            wave_number: None,
            created_at: Utc::now(),
            started_at: None,
            ended_at: None,
        }
    }

    pub fn add_player(&mut self, player_id: Uuid) -> Result<(), GameInstanceError> {
        if self.state != GameState::Lobby {
            return Err(GameInstanceError::GameNotInLobby);
        }
        if self.player_ids.len() >= self.max_players {
            return Err(GameInstanceError::GameFull);
        }
        if self.player_ids.contains(&player_id) {
            return Err(GameInstanceError::PlayerAlreadyInGame);
        }
        self.player_ids.push(player_id);
        Ok(())
    }

    pub fn remove_player(&mut self, player_id: Uuid) -> Result<(), GameInstanceError> {
        let pos = self
            .player_ids
            .iter()
            .position(|&id| id == player_id)
            .ok_or(GameInstanceError::PlayerNotInGame)?;
        self.player_ids.remove(pos);
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), GameInstanceError> {
        if self.state != GameState::Lobby {
            return Err(GameInstanceError::GameNotInLobby);
        }
        if self.player_ids.len() < 2 {
            return Err(GameInstanceError::NotEnoughPlayers);
        }
        self.state = GameState::Starting;
        self.started_at = Some(Utc::now());

        if matches!(self.mode, GameMode::TeamDeathmatch) {
            let team_count = 2;
            self.team_scores = Some(vec![0; team_count]);
        }

        if matches!(self.mode, GameMode::WavesSurvival) {
            self.wave_number = Some(0);
        }

        Ok(())
    }

    pub fn update_score(&mut self, team_index: usize, delta: i32) -> Result<(), GameInstanceError> {
        if let Some(scores) = &mut self.team_scores {
            if team_index >= scores.len() {
                return Err(GameInstanceError::InvalidTeam);
            }
            scores[team_index] += delta;
        }
        Ok(())
    }

    pub fn advance_wave(&mut self) -> Result<(), GameInstanceError> {
        if !matches!(self.mode, GameMode::WavesSurvival) {
            return Err(GameInstanceError::WrongGameMode);
        }
        let current = self.wave_number.ok_or(GameInstanceError::NoWaveActive)?;
        self.wave_number = Some(current + 1);
        Ok(())
    }

    pub fn end(&mut self) -> Result<(), GameInstanceError> {
        if self.state == GameState::Ended {
            return Err(GameInstanceError::GameAlreadyEnded);
        }
        self.state = GameState::Ended;
        self.ended_at = Some(Utc::now());
        Ok(())
    }
}

#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum GameInstanceError {
    #[error("Game is not in lobby state")]
    GameNotInLobby,
    #[error("Game is full")]
    GameFull,
    #[error("Player already in game")]
    PlayerAlreadyInGame,
    #[error("Player not in game")]
    PlayerNotInGame,
    #[error("Not enough players to start")]
    NotEnoughPlayers,
    #[error("Invalid team index")]
    InvalidTeam,
    #[error("No wave is currently active")]
    NoWaveActive,
    #[error("Wrong game mode for this operation")]
    WrongGameMode,
    #[error("Game has already ended")]
    GameAlreadyEnded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatEvent {
    pub id: Uuid,
    pub instance_id: Uuid,
    pub attacker_id: Uuid,
    pub target_id: Uuid,
    pub event_type: CombatEventType,
    pub value: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatEventType {
    Damage,
    ShieldHit,
    ArmorHit,
    Kill,
    MissileLaunch,
    MissileHit,
    ModuleActivated,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_mode_is_pvp() {
        assert!(GameMode::TeamDeathmatch.is_pvp());
        assert!(GameMode::FreeForAll.is_pvp());
        assert!(!GameMode::WavesSurvival.is_pvp());
        assert!(!GameMode::OpenWorld.is_pvp());
    }

    #[test]
    fn test_game_mode_is_pve() {
        assert!(!GameMode::TeamDeathmatch.is_pve());
        assert!(GameMode::WavesSurvival.is_pve());
        assert!(GameMode::OperationScenario.is_pve());
        assert!(!GameMode::OpenWorld.is_pve());
    }

    #[test]
    fn test_game_mode_allows_respawn() {
        assert!(GameMode::TeamDeathmatch.allows_respawn());
        assert!(GameMode::FreeForAll.allows_respawn());
        assert!(GameMode::WavesSurvival.allows_respawn());
        assert!(!GameMode::OperationScenario.allows_respawn());
        assert!(!GameMode::OpenWorld.allows_respawn());
    }

    #[test]
    fn test_game_instance_create() {
        let instance = GameInstance::new("Test Game".into(), GameMode::TeamDeathmatch, 8);

        assert_eq!(instance.name, "Test Game");
        assert_eq!(instance.mode, GameMode::TeamDeathmatch);
        assert_eq!(instance.max_players, 8);
        assert_eq!(instance.state, GameState::Lobby);
        assert!(instance.player_ids.is_empty());
    }

    #[test]
    fn test_game_instance_add_player() {
        let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 4);
        let player_id = uuid::Uuid::new_v4();

        assert!(instance.add_player(player_id).is_ok());
        assert_eq!(instance.player_ids.len(), 1);
    }

    #[test]
    fn test_game_instance_add_player_full() {
        let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 2);

        instance.add_player(uuid::Uuid::new_v4()).unwrap();
        instance.add_player(uuid::Uuid::new_v4()).unwrap();

        assert!(instance.add_player(uuid::Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_game_instance_add_player_twice() {
        let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 4);
        let player_id = uuid::Uuid::new_v4();

        instance.add_player(player_id).unwrap();
        assert!(instance.add_player(player_id).is_err());
    }

    #[test]
    fn test_game_instance_start() {
        let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 2);

        instance.add_player(uuid::Uuid::new_v4()).unwrap();
        instance.add_player(uuid::Uuid::new_v4()).unwrap();

        assert!(instance.start().is_ok());
        assert_eq!(instance.state, GameState::Starting);
        assert!(instance.started_at.is_some());
    }

    #[test]
    fn test_game_instance_start_not_enough_players() {
        let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 4);

        instance.add_player(uuid::Uuid::new_v4()).unwrap();
        assert!(instance.start().is_err());
    }

    #[test]
    fn test_game_instance_team_scores() {
        let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 4);

        instance.add_player(uuid::Uuid::new_v4()).unwrap();
        instance.add_player(uuid::Uuid::new_v4()).unwrap();
        instance.start().unwrap();

        assert!(instance.team_scores.is_some());
        let scores = instance.team_scores.unwrap();
        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0], 0);
        assert_eq!(scores[1], 0);
    }

    #[test]
    fn test_game_instance_update_score() {
        let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 4);

        instance.add_player(uuid::Uuid::new_v4()).unwrap();
        instance.add_player(uuid::Uuid::new_v4()).unwrap();
        instance.start().unwrap();

        instance.update_score(0, 10).unwrap();

        assert_eq!(instance.team_scores.unwrap()[0], 10);
    }

    #[test]
    fn test_game_instance_wave_number() {
        let mut instance = GameInstance::new("Test".into(), GameMode::WavesSurvival, 4);

        instance.add_player(uuid::Uuid::new_v4()).unwrap();
        instance.add_player(uuid::Uuid::new_v4()).unwrap();
        instance.start().unwrap();

        assert!(instance.wave_number.is_some());
        assert_eq!(instance.wave_number.unwrap(), 0);

        instance.advance_wave().unwrap();
        assert_eq!(instance.wave_number.unwrap(), 1);
    }

    #[test]
    fn test_game_instance_end() {
        let mut instance = GameInstance::new("Test".into(), GameMode::TeamDeathmatch, 2);

        instance.add_player(uuid::Uuid::new_v4()).unwrap();
        instance.add_player(uuid::Uuid::new_v4()).unwrap();
        instance.start().unwrap();

        assert!(instance.end().is_ok());
        assert_eq!(instance.state, GameState::Ended);
        assert!(instance.ended_at.is_some());
    }
}
