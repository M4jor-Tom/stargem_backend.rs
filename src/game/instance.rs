use crate::domain::{
    CombatEvent, CombatEventType, DamageResult, GameInstance, GameMode, GameState, PlayerSession,
    Ship, Weapon,
};
use crate::game::{CombatError, CombatSystem, SpecialAbilityManager, WeaponShot};
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct GameInstanceManager {
    instances: HashMap<Uuid, Arc<RwLock<GameInstance>>>,
    instance_players: HashMap<Uuid, HashMap<Uuid, PlayerState>>,
    instance_ships: HashMap<Uuid, HashMap<Uuid, Ship>>,
    player_weapons: HashMap<Uuid, Uuid>,
    combat_systems: HashMap<Uuid, CombatSystem>,
    ability_managers: HashMap<Uuid, SpecialAbilityManager>,
}

impl GameInstanceManager {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            instance_players: HashMap::new(),
            instance_ships: HashMap::new(),
            player_weapons: HashMap::new(),
            combat_systems: HashMap::new(),
            ability_managers: HashMap::new(),
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

    pub fn get_mut_combat_system(&mut self, instance_id: Uuid) -> Option<&mut CombatSystem> {
        self.combat_systems.get_mut(&instance_id)
    }

    pub fn add_ship_to_instance(&mut self, instance_id: Uuid, player_id: Uuid, ship: Ship) {
        let ships = self.instance_ships.entry(instance_id).or_default();
        ships.insert(player_id, ship);
    }

    pub fn get_ship(&self, instance_id: Uuid, player_id: Uuid) -> Option<&Ship> {
        self.instance_ships.get(&instance_id)?.get(&player_id)
    }

    pub fn get_ship_mut(&mut self, instance_id: Uuid, player_id: Uuid) -> Option<&mut Ship> {
        self.instance_ships
            .get_mut(&instance_id)?
            .get_mut(&player_id)
    }

    pub fn register_player_weapon(
        &mut self,
        instance_id: Uuid,
        player_id: Uuid,
        weapon: &Weapon,
    ) -> Option<Uuid> {
        let combat = self.combat_systems.get_mut(&instance_id)?;
        let weapon_id = combat.register_weapon(player_id, weapon);
        self.player_weapons.insert(player_id, weapon_id);
        Some(weapon_id)
    }

    pub fn fire_player_weapon(
        &mut self,
        instance_id: Uuid,
        attacker_id: Uuid,
        target_id: Uuid,
    ) -> Result<CombatResult, CombatError> {
        let weapon_id = *self
            .player_weapons
            .get(&attacker_id)
            .ok_or(CombatError::WeaponNotFound)?;

        let combat = self
            .combat_systems
            .get_mut(&instance_id)
            .ok_or(CombatError::InvalidTarget)?;

        let shot = combat.fire_weapon(weapon_id)?;

        let target_ship = self
            .instance_ships
            .get_mut(&instance_id)
            .and_then(|ships| ships.get_mut(&target_id))
            .ok_or(CombatError::InvalidTarget)?;

        let damage_result = CombatSystem::apply_damage(target_ship, &shot);

        let event = combat.create_combat_event(
            instance_id,
            attacker_id,
            target_id,
            if damage_result.hull_damage {
                CombatEventType::Kill
            } else {
                CombatEventType::Damage
            },
            shot.damage,
        );

        Ok(CombatResult {
            shot,
            damage_result,
            event,
        })
    }

    pub fn is_ship_destroyed(&self, instance_id: Uuid, player_id: Uuid) -> bool {
        self.get_ship(instance_id, player_id)
            .map(|ship| !ship.is_alive())
            .unwrap_or(true)
    }

    pub fn get_all_ships(&self, instance_id: Uuid) -> Vec<&Ship> {
        self.instance_ships
            .get(&instance_id)
            .map(|ships| ships.values().collect())
            .unwrap_or_default()
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

        for id in &ended {
            self.instances.remove(id);
            self.instance_players.remove(id);
            self.instance_ships.remove(id);
            self.combat_systems.remove(id);
            self.ability_managers.remove(id);
        }

        for player_id in ended.iter().flat_map(|id| {
            self.instance_players
                .get(id)
                .map(|p| p.keys().cloned())
                .unwrap_or_default()
        }) {
            self.player_weapons.remove(&player_id);
        }
    }
}

impl Default for GameInstanceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CombatResult {
    pub shot: WeaponShot,
    pub damage_result: DamageResult,
    pub event: CombatEvent,
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
