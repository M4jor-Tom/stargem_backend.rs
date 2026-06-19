use crate::combat::damage::DamageType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissileModelDef {
    pub id: Uuid,
    pub damage_type: DamageType,
    pub speed: f32,
    pub turn_rate: f32,
    pub lifetime_secs: f32,
    pub damage: f32,
    pub blast_radius: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_missile_model_def_construction() {
        let m = MissileModelDef {
            id: Uuid::nil(),
            damage_type: DamageType::Thermic,
            speed: 200.0,
            turn_rate: 3.0,
            lifetime_secs: 10.0,
            damage: 150.0,
            blast_radius: 5.0,
        };
        assert_eq!(m.damage_type, DamageType::Thermic);
        assert_eq!(m.speed, 200.0);
        assert_eq!(m.damage, 150.0);
    }

    #[test]
    fn test_missile_model_serde_roundtrip() {
        let m = MissileModelDef {
            id: Uuid::nil(),
            damage_type: DamageType::Kinetic,
            speed: 300.0,
            turn_rate: 5.0,
            lifetime_secs: 8.0,
            damage: 80.0,
            blast_radius: 2.0,
        };
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: MissileModelDef = serde_json::from_str(&json).unwrap();
        assert_eq!(m.damage_type, deserialized.damage_type);
        assert_eq!(m.speed, deserialized.speed);
    }
}
