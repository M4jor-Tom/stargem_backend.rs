use crate::combat::damage::DamageType;
use crate::ship::model::ShipSize;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponModel {
    Cannon,
    Beam,
    Lance,
    Autocannon,
    Pulse,
    Railgun,
    MachineGun,
    Laser,
    IonCannon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponDef {
    pub id: Uuid,
    pub name: WeaponModel,
    pub size: ShipSize,
    pub damage_type: DamageType,
    pub fire_rate: f32,
    pub damage_per_shot: f32,
    pub heat_per_shot: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_weapon_model_serde_roundtrip() {
        let variants = vec![
            WeaponModel::Cannon, WeaponModel::Beam, WeaponModel::Lance,
            WeaponModel::Autocannon, WeaponModel::Pulse, WeaponModel::Railgun,
            WeaponModel::MachineGun, WeaponModel::Laser, WeaponModel::IonCannon,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let deserialized: WeaponModel = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, deserialized);
        }
    }

    #[test]
    fn test_weapon_def_construction() {
        let def = WeaponDef {
            id: Uuid::nil(),
            name: WeaponModel::Cannon,
            size: ShipSize::Fighter,
            damage_type: DamageType::Kinetic,
            fire_rate: 2.0,
            damage_per_shot: 25.0,
            heat_per_shot: 5.0,
        };
        assert_eq!(def.name, WeaponModel::Cannon);
        assert_eq!(def.fire_rate, 2.0);
    }
}
