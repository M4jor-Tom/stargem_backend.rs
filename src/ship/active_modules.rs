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

    use crate::ship::active_module_state::{
        ActiveModuleState, ActivationFlow as StateActivationFlow, ActivationStatus,
    };

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

    #[test]
    fn test_oneshot_deducts_energy_and_starts_cooldown() {
        let mut module = ActiveModuleState::new(5.0, 30.0, StateActivationFlow::OneShot);
        assert_eq!(module.activate(100.0), Ok(30.0));
        assert!(matches!(module.status, ActivationStatus::Cooldown { .. }));
    }

    #[test]
    fn test_ongoing_toggle_starts_and_stops_drain() {
        let mut module = ActiveModuleState::new(
            5.0,
            20.0,
            StateActivationFlow::Ongoing {
                drain_per_second: 10.0,
            },
        );
        assert_eq!(module.activate(100.0), Ok(20.0));
        assert!(module.is_active());

        module.deactivate();
        assert!(matches!(module.status, ActivationStatus::Cooldown { .. }));
    }

    #[test]
    fn test_activate_rejected_when_insufficient_energy() {
        let mut module = ActiveModuleState::new(5.0, 30.0, StateActivationFlow::OneShot);
        let result = module.activate(20.0);
        assert_eq!(result, Err("insufficient energy"));
        assert!(module.is_ready());
    }

    #[test]
    fn test_activate_rejected_while_on_cooldown() {
        let mut module = ActiveModuleState::new(5.0, 30.0, StateActivationFlow::OneShot);
        module.activate(100.0).unwrap();
        let result = module.activate(100.0);
        assert_eq!(result, Err("on cooldown"));
    }

    #[test]
    fn test_cooldown_decrements_each_tick() {
        let mut module = ActiveModuleState::new(5.0, 30.0, StateActivationFlow::OneShot);
        module.activate(100.0).unwrap();

        module.update(1.0);
        if let ActivationStatus::Cooldown { remaining_secs } = &module.status {
            assert!((*remaining_secs - 4.0).abs() < f32::EPSILON);
        } else {
            panic!("expected Cooldown state");
        }

        module.update(4.0);
        assert!(module.is_ready());
    }

    #[test]
    fn test_activate_rejected_when_already_active() {
        let mut module = ActiveModuleState::new(
            5.0,
            20.0,
            StateActivationFlow::Ongoing {
                drain_per_second: 10.0,
            },
        );
        module.activate(100.0).unwrap();
        let result = module.activate(100.0);
        assert_eq!(result, Err("already active"));
    }
}
