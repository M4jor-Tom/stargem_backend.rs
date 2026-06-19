use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageMultipliers {
    pub electromagnetic: TypeMultipliers,
    pub kinetic: TypeMultipliers,
    pub thermic: TypeMultipliers,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    let raw_amount = raw_amount.max(0.0);
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

    let total_damage_dealt =
        (current_shield - shield_remaining) + (current_armor - armor_remaining);
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

    #[test]
    fn test_thermic_has_equal_multipliers() {
        let mult = DamageMultipliers::default();
        let result = apply_damage(DamageType::Thermic, 50.0, 100.0, 100.0, &mult);
        assert_eq!(result.shield_remaining, 50.0);
        assert_eq!(result.armor_remaining, 100.0);
        assert!(!result.ship_destroyed);
    }

    #[test]
    fn test_damage_bleeds_through_shield_into_armor() {
        let mult = DamageMultipliers::default();
        let result = apply_damage(DamageType::Kinetic, 200.0, 50.0, 100.0, &mult);
        assert_eq!(result.shield_remaining, 0.0);
        assert_eq!(result.armor_remaining, 25.0);
    }

    #[test]
    fn test_zero_damage_no_change() {
        let mult = DamageMultipliers::default();
        let result = apply_damage(DamageType::Electromagnetic, 0.0, 100.0, 100.0, &mult);
        assert_eq!(result.shield_remaining, 100.0);
        assert_eq!(result.armor_remaining, 100.0);
        assert!(!result.ship_destroyed);
    }

    #[test]
    fn test_exact_shield_absorption() {
        let mult = DamageMultipliers::default();
        let result = apply_damage(DamageType::Thermic, 100.0, 100.0, 100.0, &mult);
        assert_eq!(result.shield_remaining, 0.0);
        assert_eq!(result.armor_remaining, 100.0);
        assert!(!result.ship_destroyed);
    }

    #[test]
    fn test_overkill_through_armor() {
        let mult = DamageMultipliers::default();
        let result = apply_damage(DamageType::Thermic, 500.0, 0.0, 100.0, &mult);
        assert!(result.ship_destroyed);
        assert_eq!(result.armor_remaining, 0.0);
        assert_eq!(result.mitigated, 400.0);
    }

    #[test]
    fn test_negative_damage_clamped_to_zero() {
        let mult = DamageMultipliers::default();
        let result = apply_damage(DamageType::Thermic, -50.0, 100.0, 100.0, &mult);
        assert!((result.shield_remaining - 100.0).abs() < 1e-4, "negative damage should not heal shield");
        assert!((result.armor_remaining - 100.0).abs() < 1e-4, "negative damage should not heal armor");
    }

    #[test]
    fn test_all_damage_types_with_zero_shield() {
        let mult = DamageMultipliers::default();

        let em = apply_damage(DamageType::Electromagnetic, 100.0, 0.0, 100.0, &mult);
        assert!((em.armor_remaining - 25.0).abs() < 1e-4, "EM: 100*1.5 shield → 150*0.5 armor = 75 damage, 25 remaining");

        let kin = apply_damage(DamageType::Kinetic, 100.0, 0.0, 100.0, &mult);
        assert!((kin.armor_remaining - 25.0).abs() < 1e-4, "Kinetic: 100*0.5 shield → 50*1.5 armor = 75 damage, 25 remaining");

        let therm = apply_damage(DamageType::Thermic, 50.0, 0.0, 100.0, &mult);
        assert!((therm.armor_remaining - 50.0).abs() < 1e-4, "Thermic: 50*1.0 shield → 50*1.0 armor = 50 damage, 50 remaining");
    }

    #[test]
    fn test_load_damage_multipliers_missing_file_returns_defaults() {
        let result = load_damage_multipliers("/nonexistent/path/to/file.toml");
        assert_eq!(result, DamageMultipliers::default());
    }

    #[test]
    fn test_load_damage_multipliers_valid_file() {
        let toml_content = r#"
[electromagnetic]
shield = 2.0
armor = 0.25

[kinetic]
shield = 0.25
armor = 2.0

[thermic]
shield = 1.0
armor = 1.0
"#;
        let dir = std::env::temp_dir();
        let path = dir.join("test_damage_mult.toml");
        std::fs::write(&path, toml_content).unwrap();

        let result = load_damage_multipliers(path.to_str().unwrap());
        std::fs::remove_file(&path).unwrap();

        assert!((result.electromagnetic.shield - 2.0).abs() < 1e-6);
        assert!((result.electromagnetic.armor - 0.25).abs() < 1e-6);
        assert!((result.kinetic.shield - 0.25).abs() < 1e-6);
        assert!((result.kinetic.armor - 2.0).abs() < 1e-6);
    }
}
