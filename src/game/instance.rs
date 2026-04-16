use crate::domain::{GameInstance, GameMode, GameState, PlayerSession, Position};
use crate::game::{CombatError, CombatSystem, SpecialAbilityManager};
use crate::network::SessionManager;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct GameInstanceManager {
    instances: HashMap<Uuid, Arc<RwLock<GameInstance>>>,
    instance_players: HashMap<Uuid, HashMap<Uuid, PlayerState>>,
    combat_systems: HashMap<Uuid, CombatSystem>,
    ability_managers: HashMap<Uuid, SpecialAbilityManager>,
    session_manager: Arc<SessionManager>,
}

impl GameInstanceManager {
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self {
            instances: HashMap::new(),
            instance_players: HashMap::new(),
            combat_systems: HashMap::new(),
            ability_managers: HashMap::new(),
            session_manager,
        }
    }

    pub fn create_instance(
        &mut self,
        name: String,
        mode: GameMode,
        max_players: usize,
    ) -> Arc<RwLock<GameInstance>> {
        let instance = GameInstance::new(name, mode, max_players);
        let id = instance.id;

        let arc = Arc::new(RwLock::new(instance));
        self.instances.insert(id, arc.clone());

        self.combat_systems.insert(id, CombatSystem::new());
        self.ability_managers
            .insert(id, SpecialAbilityManager::new());

        arc
    }

    pub fn get_instance(&self, id: Uuid) -> Option<Arc<RwLock<GameInstance>>> {
        self.instances.get(&id).cloned()
    }

    pub fn join_instance(
        &mut self,
        instance_id: Uuid,
        player_id: Uuid,
        session: PlayerSession,
    ) -> Result<(), GameInstanceError> {
        let instance = self
            .instances
            .get(&instance_id)
            .ok_or(GameInstanceError::InstanceNotFound)?;

        let mut inst = instance.write();
        inst.add_player(player_id)?;

        let players = self.instance_players.entry(instance_id).or_default();
        players.insert(
            player_id,
            PlayerState {
                session,
                last_activity: Utc::now(),
            },
        );

        Ok(())
    }

    pub fn leave_instance(
        &mut self,
        instance_id: Uuid,
        player_id: Uuid,
    ) -> Result<(), GameInstanceError> {
        let instance = self
            .instances
            .get(&instance_id)
            .ok_or(GameInstanceError::InstanceNotFound)?;

        let mut inst = instance.write();
        inst.remove_player(player_id)?;

        if let Some(players) = self.instance_players.get_mut(&instance_id) {
            players.remove(&player_id);
        }

        Ok(())
    }

    pub fn start_instance(&self, instance_id: Uuid) -> Result<(), GameInstanceError> {
        let instance = self
            .instances
            .get(&instance_id)
            .ok_or(GameInstanceError::InstanceNotFound)?;

        let mut inst = instance.write();
        inst.start()?;

        Ok(())
    }

    pub fn end_instance(&self, instance_id: Uuid) -> Result<(), GameInstanceError> {
        let instance = self
            .instances
            .get(&instance_id)
            .ok_or(GameInstanceError::InstanceNotFound)?;

        let mut inst = instance.write();
        inst.end()?;

        Ok(())
    }

    pub fn get_players_in_instance(
        &self,
        instance_id: Uuid,
    ) -> Option<&HashMap<Uuid, PlayerState>> {
        self.instance_players.get(&instance_id)
    }

    pub fn get_combat_system(&self, instance_id: Uuid) -> Option<&CombatSystem> {
        self.combat_systems.get(&instance_id)
    }

    pub fn get_ability_manager(&self, instance_id: Uuid) -> Option<&SpecialAbilityManager> {
        self.ability_managers.get(&instance_id)
    }

    pub fn list_instances(&self, mode: Option<GameMode>) -> Vec<Arc<RwLock<GameInstance>>> {
        self.instances
            .values()
            .filter(|inst| {
                let state = inst.read().state;
                mode.map_or(state == GameState::Lobby, |m| {
                    let instance = inst.read();
                    instance.mode == m && instance.state == GameState::Lobby
                })
            })
            .cloned()
            .collect()
    }

    pub fn cleanup_ended(&mut self) {
        let ended: Vec<Uuid> = self
            .instances
            .iter()
            .filter(|(_, inst)| inst.read().state == GameState::Ended)
            .map(|(id, _)| *id)
            .collect();

        for id in ended {
            self.instances.remove(&id);
            self.instance_players.remove(&id);
            self.combat_systems.remove(&id);
            self.ability_managers.remove(&id);
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub session: PlayerSession,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum GameInstanceError {
    #[error("Instance not found")]
    InstanceNotFound,
    #[error("Instance is full")]
    InstanceFull,
    #[error("Player already in instance")]
    PlayerAlreadyIn,
    #[error("Player not in instance")]
    PlayerNotIn,
    #[error("Instance not in lobby")]
    NotInLobby,
    #[error("{0}")]
    Other(String),
}

impl From<crate::domain::GameInstanceError> for GameInstanceError {
    fn from(e: crate::domain::GameInstanceError) -> Self {
        GameInstanceError::Other(e.to_string())
    }
}
