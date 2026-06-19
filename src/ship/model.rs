use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShipSize {
    Frigate,
    Fighter,
    Interceptor,
}

impl ShipSize {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "frigate" => Some(Self::Frigate),
            "fighter" => Some(Self::Fighter),
            "interceptor" => Some(Self::Interceptor),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShipRole {
    Engineer,
    LongRange,
    Guard,
    Tackler,
    Gunship,
    Command,
    CoverOps,
    Recon,
    Ecm,
}

impl ShipRole {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "engineer" => Some(Self::Engineer),
            "longrange" => Some(Self::LongRange),
            "guard" => Some(Self::Guard),
            "tackler" => Some(Self::Tackler),
            "gunship" => Some(Self::Gunship),
            "command" => Some(Self::Command),
            "coverops" => Some(Self::CoverOps),
            "recon" => Some(Self::Recon),
            "ecm" => Some(Self::Ecm),
            _ => None,
        }
    }

    pub fn special_module(&self) -> &'static str {
        match self {
            Self::Engineer => "drones",
            Self::LongRange => "sniper_weapon",
            Self::Guard => "phasic_shield",
            Self::Tackler => "cloak",
            Self::Gunship => "overclock",
            Self::Command => "command_shield",
            Self::CoverOps => "plasma_web",
            Self::Recon => "hyper_propulsion",
            Self::Ecm => "electromagnetic_surge",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HullStats {
    pub base_shield: f32,
    pub base_armor: f32,
    pub base_energy: f32,
    pub base_speed: f32,
    pub base_agility: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipModel {
    pub id: Uuid,
    pub size: ShipSize,
    pub role: ShipRole,
    pub price: i32,
    pub base_stats: HullStats,
    pub shields_count: u8,
    pub armors_count: u8,
    pub capacitors_count: u8,
    pub motors_count: u8,
    pub computers_count: u8,
}

#[derive(Debug, Clone)]
pub struct PlayerShip {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ship_model_id: Uuid,
    pub loadout_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_ship_size_from_str_valid() {
        assert_eq!(ShipSize::from_str("frigate"), Some(ShipSize::Frigate));
        assert_eq!(ShipSize::from_str("Frigate"), Some(ShipSize::Frigate));
        assert_eq!(ShipSize::from_str("fighter"), Some(ShipSize::Fighter));
        assert_eq!(ShipSize::from_str("interceptor"), Some(ShipSize::Interceptor));
    }

    #[test]
    fn test_ship_size_from_str_invalid() {
        assert_eq!(ShipSize::from_str("dreadnought"), None);
        assert_eq!(ShipSize::from_str(""), None);
    }

    #[test]
    fn test_ship_role_from_str_all_variants() {
        assert_eq!(ShipRole::from_str("engineer"), Some(ShipRole::Engineer));
        assert_eq!(ShipRole::from_str("longrange"), Some(ShipRole::LongRange));
        assert_eq!(ShipRole::from_str("guard"), Some(ShipRole::Guard));
        assert_eq!(ShipRole::from_str("tackler"), Some(ShipRole::Tackler));
        assert_eq!(ShipRole::from_str("gunship"), Some(ShipRole::Gunship));
        assert_eq!(ShipRole::from_str("command"), Some(ShipRole::Command));
        assert_eq!(ShipRole::from_str("coverops"), Some(ShipRole::CoverOps));
        assert_eq!(ShipRole::from_str("recon"), Some(ShipRole::Recon));
        assert_eq!(ShipRole::from_str("ecm"), Some(ShipRole::Ecm));
    }

    #[test]
    fn test_ship_role_from_str_case_insensitive() {
        assert_eq!(ShipRole::from_str("Engineer"), Some(ShipRole::Engineer));
        assert_eq!(ShipRole::from_str("LONGRANGE"), Some(ShipRole::LongRange));
        assert_eq!(ShipRole::from_str("CoverOps"), Some(ShipRole::CoverOps));
    }

    #[test]
    fn test_ship_role_from_str_invalid() {
        assert_eq!(ShipRole::from_str("unknown"), None);
        assert_eq!(ShipRole::from_str(""), None);
    }

    #[test]
    fn test_ship_size_parsing_all_variants() {
        assert_eq!(ShipSize::from_str("Frigate"), Some(ShipSize::Frigate));
        assert_eq!(ShipSize::from_str("Fighter"), Some(ShipSize::Fighter));
        assert_eq!(ShipSize::from_str("Interceptor"), Some(ShipSize::Interceptor));
    }

    #[test]
    fn test_ship_size_case_insensitivity() {
        assert_eq!(ShipSize::from_str("FRIGATE"), Some(ShipSize::Frigate));
        assert_eq!(ShipSize::from_str("frigate"), Some(ShipSize::Frigate));
        assert_eq!(ShipSize::from_str("FrIgAtE"), Some(ShipSize::Frigate));
    }

    #[test]
    fn test_ship_size_invalid_inputs() {
        assert_eq!(ShipSize::from_str(""), None);
        assert_eq!(ShipSize::from_str(" "), None);
        assert_eq!(ShipSize::from_str("dreadnought"), None);
        assert_eq!(ShipSize::from_str("123"), None);
    }

    #[test]
    fn test_ship_size_serde_roundtrip() {
        let original = ShipSize::Frigate;
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ShipSize = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_ship_role_serde_roundtrip_all_variants() {
        let roles = vec![
            ShipRole::Command,
            ShipRole::CoverOps,
            ShipRole::Ecm,
            ShipRole::Engineer,
            ShipRole::Guard,
            ShipRole::Gunship,
            ShipRole::LongRange,
            ShipRole::Recon,
            ShipRole::Tackler,
        ];
        for role in roles {
            let json = serde_json::to_string(&role).unwrap();
            let deserialized: ShipRole = serde_json::from_str(&json).unwrap();
            assert_eq!(role, deserialized, "failed roundtrip for {:?}", role);
        }
    }

    #[test]
    fn test_ship_role_special_module_all_variants() {
        assert_eq!(ShipRole::Engineer.special_module(), "drones");
        assert_eq!(ShipRole::LongRange.special_module(), "sniper_weapon");
        assert_eq!(ShipRole::Guard.special_module(), "phasic_shield");
        assert_eq!(ShipRole::Tackler.special_module(), "cloak");
        assert_eq!(ShipRole::Gunship.special_module(), "overclock");
        assert_eq!(ShipRole::Command.special_module(), "command_shield");
        assert_eq!(ShipRole::CoverOps.special_module(), "plasma_web");
        assert_eq!(ShipRole::Recon.special_module(), "hyper_propulsion");
        assert_eq!(ShipRole::Ecm.special_module(), "electromagnetic_surge");
    }
}
