use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivationFlow {
    OneShot,
    Ongoing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveModule {
    pub id: String,
    pub name: String,
    pub energy_cost: f32,
    pub cooldown_secs: f32,
    pub activation_flow: ActivationFlow,
    pub effect_definition: String,
}

pub const MAX_ACTIVE_MODULES: usize = 4;
