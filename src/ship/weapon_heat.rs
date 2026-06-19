use serde::{Deserialize, Serialize};

const DEFAULT_MAX_HEAT: f32 = 100.0;
const DEFAULT_OVERHEAT_THRESHOLD: f32 = 100.0;
const DEFAULT_COOLDOWN_RATE: f32 = 25.0;
const DEFAULT_OVERHEAT_COOLDOWN_RATE: f32 = 50.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponHeatState {
    pub current_heat: f32,
    pub max_heat: f32,
    pub overheat_threshold: f32,
    pub cooldown_rate: f32,
    pub overheat_cooldown_rate: f32,
    pub is_overheated: bool,
}

impl Default for WeaponHeatState {
    fn default() -> Self {
        Self {
            current_heat: 0.0,
            max_heat: DEFAULT_MAX_HEAT,
            overheat_threshold: DEFAULT_OVERHEAT_THRESHOLD,
            cooldown_rate: DEFAULT_COOLDOWN_RATE,
            overheat_cooldown_rate: DEFAULT_OVERHEAT_COOLDOWN_RATE,
            is_overheated: false,
        }
    }
}

impl WeaponHeatState {
    pub fn new(max_heat: f32, cooldown_rate: f32, overheat_cooldown_rate: f32) -> Self {
        Self {
            current_heat: 0.0,
            max_heat,
            overheat_threshold: max_heat,
            cooldown_rate,
            overheat_cooldown_rate,
            is_overheated: false,
        }
    }

    pub fn fire(&mut self, heat_per_shot: f32) -> bool {
        if self.is_overheated {
            return false;
        }
        self.current_heat = (self.current_heat + heat_per_shot).min(self.max_heat);
        if self.current_heat >= self.overheat_threshold {
            self.is_overheated = true;
        }
        true
    }

    pub fn update(&mut self, dt: f32) {
        let rate = if self.is_overheated {
            self.overheat_cooldown_rate
        } else {
            self.cooldown_rate
        };
        self.current_heat = (self.current_heat - rate * dt).max(0.0);
        if self.current_heat < self.overheat_threshold {
            self.is_overheated = false;
        }
    }

    pub fn heat_percentage(&self) -> f32 {
        if self.max_heat <= 0.0 {
            return 0.0;
        }
        (self.current_heat / self.max_heat) * 100.0
    }

    pub fn can_fire(&self) -> bool {
        !self.is_overheated
    }
}
