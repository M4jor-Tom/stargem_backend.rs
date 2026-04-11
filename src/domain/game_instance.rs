use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
        matches!(self, GameMode::TeamDeathmatch | GameMode::FreeForAll | GameMode::WavesSurvival)
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
        let pos = self.player_ids.iter().position(|&id| id == player_id)
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
