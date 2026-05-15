use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    Electromagnetic,
    Kinetic,
    Thermic,
}

impl DamageType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "electromagnetic" => Some(Self::Electromagnetic),
            "kinetic" => Some(Self::Kinetic),
            "thermic" => Some(Self::Thermic),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageMultipliers {
    pub electromagnetic: TypeMultipliers,
    pub kinetic: TypeMultipliers,
    pub thermic: TypeMultipliers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMultipliers {
    pub shield: f32,
    pub armor: f32,
}

impl Default for DamageMultipliers {
    fn default() -> Self {
        Self {
            electromagnetic: TypeMultipliers {
                shield: 1.5,
                armor: 0.5,
            },
            kinetic: TypeMultipliers {
                shield: 0.5,
                armor: 1.5,
            },
            thermic: TypeMultipliers {
                shield: 1.0,
                armor: 1.0,
            },
        }
    }
}

pub fn load_damage_multipliers(path: &str) -> DamageMultipliers {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}

pub struct DamageResult {
    pub raw_amount: f32,
    pub shield_damage: f32,
    pub armor_damage: f32,
    pub mitigated: f32,
    pub shield_remaining: f32,
    pub armor_remaining: f32,
    pub ship_destroyed: bool,
}

pub fn apply_damage(
    damage_type: DamageType,
    raw_amount: f32,
    current_shield: f32,
    current_armor: f32,
    multipliers: &DamageMultipliers,
) -> DamageResult {
    let mult = match damage_type {
        DamageType::Electromagnetic => &multipliers.electromagnetic,
        DamageType::Kinetic => &multipliers.kinetic,
        DamageType::Thermic => &multipliers.thermic,
    };

    let shield_damage = raw_amount * mult.shield;
    let mut remaining = shield_damage;
    let mut shield_remaining = current_shield;
    let mut armor_remaining = current_armor;

    if remaining <= shield_remaining {
        shield_remaining -= remaining;
        remaining = 0.0;
    } else {
        remaining -= shield_remaining;
        shield_remaining = 0.0;
    }

    if remaining > 0.0 {
        let armor_damage = remaining * mult.armor;
        armor_remaining = (armor_remaining - armor_damage).max(0.0);
    }

    let total_damage_dealt = (current_shield - shield_remaining) + (current_armor - armor_remaining);
    let mitigated = (raw_amount - total_damage_dealt).max(0.0);

    DamageResult {
        raw_amount,
        shield_damage: current_shield - shield_remaining,
        armor_damage: current_armor - armor_remaining,
        mitigated,
        shield_remaining,
        armor_remaining,
        ship_destroyed: armor_remaining <= 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electromagnetic_vs_shield() {
        let mult = DamageMultipliers::default();
        let result = apply_damage(DamageType::Electromagnetic, 100.0, 100.0, 100.0, &mult);
        assert_eq!(result.shield_remaining, 0.0);
        assert_eq!(result.armor_remaining, 75.0);
    }

    #[test]
    fn test_ship_destroyed() {
        let mult = DamageMultipliers::default();
        let result = apply_damage(DamageType::Kinetic, 200.0, 0.0, 100.0, &mult);
        assert!(result.ship_destroyed);
    }
}
