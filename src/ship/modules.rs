use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PassiveModuleType {
    Shield,
    Armor,
    Capacitor,
    Motor,
    Computer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveModule {
    pub id: String,
    pub name: String,
    pub module_type: PassiveModuleType,
    pub stat_modifiers: HashMap<String, f32>,
}
