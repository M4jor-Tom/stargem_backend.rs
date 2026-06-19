use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivationStatus {
    Ready,
    Active { ongoing_drain_per_sec: f32 },
    Cooldown { remaining_secs: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivationFlow {
    OneShot,
    Ongoing { drain_per_second: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveModuleState {
    pub cooldown_secs: f32,
    pub energy_cost: f32,
    pub activation_type: ActivationFlow,
    pub status: ActivationStatus,
}

impl ActiveModuleState {
    pub fn new(cooldown_secs: f32, energy_cost: f32, activation_type: ActivationFlow) -> Self {
        Self {
            cooldown_secs,
            energy_cost,
            activation_type,
            status: ActivationStatus::Ready,
        }
    }

    pub fn activate(&mut self, current_energy: f32) -> Result<f32, &'static str> {
        match &self.status {
            ActivationStatus::Ready => {
                if current_energy < self.energy_cost {
                    return Err("insufficient energy");
                }
                match &self.activation_type {
                    ActivationFlow::OneShot => {
                        self.status = ActivationStatus::Cooldown {
                            remaining_secs: self.cooldown_secs,
                        };
                        Ok(self.energy_cost)
                    }
                    ActivationFlow::Ongoing { .. } => {
                        self.status = ActivationStatus::Active {
                            ongoing_drain_per_sec: self.energy_cost,
                        };
                        Ok(self.energy_cost)
                    }
                }
            }
            ActivationStatus::Active { .. } => Err("already active"),
            ActivationStatus::Cooldown { .. } => Err("on cooldown"),
        }
    }

    pub fn deactivate(&mut self) {
        if matches!(self.status, ActivationStatus::Active { .. }) {
            self.status = ActivationStatus::Cooldown {
                remaining_secs: self.cooldown_secs,
            };
        }
    }

    pub fn update(&mut self, dt: f32) {
        match self.status {
            ActivationStatus::Cooldown { remaining_secs } => {
                let new_remaining = remaining_secs - dt;
                if new_remaining <= 0.0 {
                    self.status = ActivationStatus::Ready;
                } else {
                    self.status = ActivationStatus::Cooldown {
                        remaining_secs: new_remaining,
                    };
                }
            }
            _ => {}
        }
    }

    pub fn is_ready(&self) -> bool {
        self.status == ActivationStatus::Ready
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, ActivationStatus::Active { .. })
    }
}
