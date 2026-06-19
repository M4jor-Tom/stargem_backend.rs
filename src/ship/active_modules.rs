use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivationFlow {
    OneShot,
    Ongoing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveModuleDef {
    pub id: Uuid,
    pub model: i32,
    pub energy_cost: f32,
    pub cooldown_seconds: f32,
    pub activation_flow: ActivationFlow,
}

pub const MAX_ACTIVE_MODULES: usize = 4;

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_max_active_modules_constant() {
        assert_eq!(MAX_ACTIVE_MODULES, 4);
    }

    #[test]
    fn test_activation_flow_serde() {
        let json = serde_json::to_string(&ActivationFlow::OneShot).unwrap();
        assert_eq!(json, "\"one_shot\"");
        let json = serde_json::to_string(&ActivationFlow::Ongoing).unwrap();
        assert_eq!(json, "\"ongoing\"");
    }

    #[test]
    fn test_active_module_def_construction() {
        let def = ActiveModuleDef {
            id: Uuid::nil(),
            model: 1,
            energy_cost: 25.0,
            cooldown_seconds: 10.0,
            activation_flow: ActivationFlow::OneShot,
        };
        assert_eq!(def.energy_cost, 25.0);
        assert_eq!(def.activation_flow, ActivationFlow::OneShot);
    }
}
