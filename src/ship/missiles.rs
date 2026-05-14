use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Missile {
    pub id: String,
    pub name: String,
    pub speed: f32,
    pub turn_rate: f32,
    pub lifetime_secs: f32,
    pub lock_on_range: f32,
    pub damage: f32,
    pub blast_radius: f32,
}
