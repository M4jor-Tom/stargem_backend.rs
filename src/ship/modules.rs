use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassiveModuleType {
    Shield,
    Armor,
    Capacitor,
    Motor,
    Computer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveModuleDef {
    pub id: Uuid,
    pub model: i32,
    pub module_type: PassiveModuleType,
    pub shield_hp_modifier: f32,
    pub armor_hp_modifier: f32,
    pub energy_modifier: f32,
    pub speed_modifier: f32,
    pub agility_modifier: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_passive_module_type_serde_roundtrip() {
        let variants = vec![
            PassiveModuleType::Shield, PassiveModuleType::Armor,
            PassiveModuleType::Capacitor, PassiveModuleType::Motor,
            PassiveModuleType::Computer,
        ];
        for v in &variants {
            let json = serde_json::to_string(v).unwrap();
            let deserialized: PassiveModuleType = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, deserialized);
        }
    }

    #[test]
    fn test_passive_module_def_construction() {
        let def = PassiveModuleDef {
            id: Uuid::nil(),
            model: 1,
            module_type: PassiveModuleType::Shield,
            shield_hp_modifier: 0.1,
            armor_hp_modifier: 0.0,
            energy_modifier: -0.05,
            speed_modifier: 0.0,
            agility_modifier: 0.0,
        };
        assert_eq!(def.model, 1);
        assert_eq!(def.module_type, PassiveModuleType::Shield);
    }
}
