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
    use crate::ship::weapon_heat::WeaponHeatState;
    use uuid::Uuid;

    #[test]
    fn test_heat_accumulates_on_fire() {
        let mut heat = WeaponHeatState::new(100.0, 25.0, 50.0);
        assert!(heat.fire(20.0));
        assert!((heat.current_heat - 20.0).abs() < f32::EPSILON);
        assert!(!heat.is_overheated);
    }

    #[test]
    fn test_overheat_threshold_blocks_fire() {
        let mut heat = WeaponHeatState::new(100.0, 25.0, 50.0);
        for _ in 0..5 {
            heat.fire(20.0);
        }
        assert!(heat.is_overheated);
        assert!(!heat.can_fire());
        assert!(!heat.fire(20.0));
    }

    #[test]
    fn test_cooldown_reduces_heat_when_not_firing() {
        let mut heat = WeaponHeatState::new(100.0, 25.0, 50.0);
        heat.fire(50.0);
        assert!((heat.current_heat - 50.0).abs() < f32::EPSILON);

        heat.update(1.0);
        assert!((heat.current_heat - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_overheat_forces_cooldown() {
        let mut heat = WeaponHeatState::new(100.0, 10.0, 50.0);
        for _ in 0..10 {
            heat.fire(10.0);
        }
        assert!(heat.is_overheated);

        heat.update(0.5);
        assert!((heat.current_heat - 75.0).abs() < f32::EPSILON);
        assert!(!heat.is_overheated);
        assert!(heat.can_fire());
    }

    #[test]
    fn test_heat_clamps_to_range() {
        let mut heat = WeaponHeatState::new(100.0, 25.0, 50.0);
        for _ in 0..10 {
            heat.fire(20.0);
        }
        assert!((heat.current_heat - 100.0).abs() < f32::EPSILON);

        heat.update(10.0);
        assert!((heat.current_heat - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_heat_percentage() {
        let mut heat = WeaponHeatState::new(100.0, 25.0, 50.0);
        assert!((heat.heat_percentage() - 0.0).abs() < f32::EPSILON);

        heat.fire(50.0);
        assert!((heat.heat_percentage() - 50.0).abs() < 0.001);

        heat.fire(50.0);
        assert!((heat.heat_percentage() - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_can_fire_when_not_overheated() {
        let heat = WeaponHeatState::new(100.0, 25.0, 50.0);
        assert!(heat.can_fire());
    }

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
