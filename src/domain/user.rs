use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MAX_HANGAR_SIZE: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub credits: i64,
    pub created_at: DateTime<Utc>,
    pub last_login: DateTime<Utc>,
}

impl User {
    pub fn new(username: String, email: String, password_hash: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            email,
            password_hash,
            credits: 1000,
            created_at: Utc::now(),
            last_login: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hangar {
    pub user_id: Uuid,
    pub ship_ids: Vec<Uuid>,
    pub selected_ship_index: Option<usize>,
}

impl Hangar {
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            ship_ids: Vec::new(),
            selected_ship_index: None,
        }
    }

    pub fn add_ship(&mut self, ship_id: Uuid) -> Result<(), HangarError> {
        if self.ship_ids.len() >= MAX_HANGAR_SIZE {
            return Err(HangarError::HangarFull);
        }
        if self.ship_ids.contains(&ship_id) {
            return Err(HangarError::ShipAlreadyInHangar);
        }
        self.ship_ids.push(ship_id);
        Ok(())
    }

    pub fn remove_ship(&mut self, ship_id: Uuid) -> Result<(), HangarError> {
        let pos = self
            .ship_ids
            .iter()
            .position(|&id| id == ship_id)
            .ok_or(HangarError::ShipNotInHangar)?;
        self.ship_ids.remove(pos);

        if let Some(idx) = self.selected_ship_index {
            if idx >= self.ship_ids.len() {
                self.selected_ship_index = None;
            }
        }
        Ok(())
    }

    pub fn select_ship(&mut self, index: usize) -> Result<Uuid, HangarError> {
        if index >= self.ship_ids.len() {
            return Err(HangarError::InvalidShipIndex);
        }
        self.selected_ship_index = Some(index);
        Ok(self.ship_ids[index])
    }

    pub fn selected_ship(&self) -> Option<Uuid> {
        self.selected_ship_index.map(|i| self.ship_ids[i])
    }
}

#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum HangarError {
    #[error("Hangar is full (max {MAX_HANGAR_SIZE} ships)")]
    HangarFull,
    #[error("Ship is already in hangar")]
    ShipAlreadyInHangar,
    #[error("Ship is not in hangar")]
    ShipNotInHangar,
    #[error("Invalid ship index")]
    InvalidShipIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceStation {
    pub id: Uuid,
    pub name: String,
    pub system_id: Uuid,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub current_ship_id: Uuid,
    pub position: Position,
    pub rotation: f32,
    pub velocity: Position,
    pub docked_at: Option<Uuid>,
    pub game_instance_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl PlayerSession {
    pub fn new(user_id: Uuid, ship_id: Uuid, station_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            current_ship_id: ship_id,
            position: Position::default(),
            rotation: 0.0,
            velocity: Position::default(),
            docked_at: Some(station_id),
            game_instance_id: None,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod hangar_tests {
    use super::*;

    #[test]
    fn test_hangar_add_ship() {
        let mut hangar = Hangar::new(uuid::Uuid::new_v4());
        let ship_id = uuid::Uuid::new_v4();

        assert!(hangar.add_ship(ship_id).is_ok());
        assert_eq!(hangar.ship_ids.len(), 1);
    }

    #[test]
    fn test_hangar_add_ship_already_exists() {
        let mut hangar = Hangar::new(uuid::Uuid::new_v4());
        let ship_id = uuid::Uuid::new_v4();

        hangar.add_ship(ship_id).unwrap();
        let result = hangar.add_ship(ship_id);

        assert!(result.is_err());
        assert_eq!(hangar.ship_ids.len(), 1);
    }

    #[test]
    fn test_hangar_max_capacity() {
        let mut hangar = Hangar::new(uuid::Uuid::new_v4());

        for _ in 0..MAX_HANGAR_SIZE {
            let ship_id = uuid::Uuid::new_v4();
            assert!(hangar.add_ship(ship_id).is_ok());
        }

        let extra_ship = uuid::Uuid::new_v4();
        assert!(hangar.add_ship(extra_ship).is_err());
    }

    #[test]
    fn test_hangar_remove_ship() {
        let mut hangar = Hangar::new(uuid::Uuid::new_v4());
        let ship_id = uuid::Uuid::new_v4();

        hangar.add_ship(ship_id).unwrap();
        assert!(hangar.remove_ship(ship_id).is_ok());
        assert!(hangar.ship_ids.is_empty());
    }

    #[test]
    fn test_hangar_remove_ship_not_found() {
        let mut hangar = Hangar::new(uuid::Uuid::new_v4());
        let ship_id = uuid::Uuid::new_v4();

        assert!(hangar.remove_ship(ship_id).is_err());
    }

    #[test]
    fn test_hangar_select_ship() {
        let mut hangar = Hangar::new(uuid::Uuid::new_v4());
        let ship_id1 = uuid::Uuid::new_v4();
        let ship_id2 = uuid::Uuid::new_v4();

        hangar.add_ship(ship_id1).unwrap();
        hangar.add_ship(ship_id2).unwrap();

        assert_eq!(hangar.select_ship(0).unwrap(), ship_id1);
        assert_eq!(hangar.select_ship(1).unwrap(), ship_id2);
        assert_eq!(hangar.selected_ship(), Some(ship_id2));
    }

    #[test]
    fn test_hangar_select_invalid_index() {
        let mut hangar = Hangar::new(uuid::Uuid::new_v4());

        assert!(hangar.select_ship(0).is_err());
    }

    #[test]
    fn test_hangar_select_ship_updates_on_remove() {
        let mut hangar = Hangar::new(uuid::Uuid::new_v4());
        let ship_id1 = uuid::Uuid::new_v4();
        let ship_id2 = uuid::Uuid::new_v4();

        hangar.add_ship(ship_id1).unwrap();
        hangar.add_ship(ship_id2).unwrap();
        hangar.select_ship(1).unwrap();

        hangar.remove_ship(ship_id2).unwrap();
        assert!(hangar.selected_ship_index.is_none());
    }
}
