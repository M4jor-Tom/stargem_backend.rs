use crate::ship::stats::PlayerShipStats;

#[derive(Debug, Clone)]
pub struct PhysicsState {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ShipInput {
    pub throttle: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl PhysicsState {
    pub fn new() -> Self {
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
        }
    }

    pub fn update(&mut self, input: &ShipInput, stats: &PlayerShipStats, dt: f32) {
        let speed_cap = stats.speed;
        let drag = 2.0;

        for component in self.velocity.iter_mut() {
            *component *= f32::exp(-drag * dt);
        }

        let thrust = input.throttle * speed_cap * 3.0;
        let forward = self.forward_vector();
        self.velocity[0] += forward[0] * thrust * dt;
        self.velocity[1] += forward[1] * thrust * dt;
        self.velocity[2] += forward[2] * thrust * dt;

        let current_speed =
            (self.velocity[0].powi(2) + self.velocity[1].powi(2) + self.velocity[2].powi(2)).sqrt();
        if current_speed > speed_cap {
            let scale = speed_cap / current_speed;
            self.velocity[0] *= scale;
            self.velocity[1] *= scale;
            self.velocity[2] *= scale;
        }

        self.position[0] += self.velocity[0] * dt;
        self.position[1] += self.velocity[1] * dt;
        self.position[2] += self.velocity[2] * dt;

        let agility = stats.agility;
        self.apply_rotation(
            input.yaw * agility * dt,
            input.pitch * agility * dt,
            input.roll * agility * dt,
        );
    }

    fn forward_vector(&self) -> [f32; 3] {
        let [x, y, z, w] = self.rotation;
        [
            2.0 * (x * z + w * y),
            2.0 * (y * z - w * x),
            1.0 - 2.0 * (x * x + y * y),
        ]
    }

    fn apply_rotation(&mut self, yaw: f32, pitch: f32, roll: f32) {
        let qy = quaternion_from_axis_angle([0.0, 1.0, 0.0], yaw);
        let qp = quaternion_from_axis_angle([1.0, 0.0, 0.0], pitch);
        let qr = quaternion_from_axis_angle([0.0, 0.0, 1.0], roll);
        self.rotation = quaternion_multiply(
            &self.rotation,
            &quaternion_multiply(&qy, &quaternion_multiply(&qp, &qr)),
        );
    }
}

fn quaternion_from_axis_angle(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let half = angle * 0.5;
    let s = half.sin();
    [axis[0] * s, axis[1] * s, axis[2] * s, half.cos()]
}

fn quaternion_multiply(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
    [
        a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
        a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
        a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
        a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ship::stats::PlayerShipStats;

    fn dummy_stats() -> PlayerShipStats {
        PlayerShipStats {
            max_shield: 100.0, max_armor: 100.0, max_energy: 100.0,
            speed: 50.0, agility: 10.0,
            current_shield: 100.0, current_armor: 100.0, current_energy: 100.0,
        }
    }

    #[test]
    fn test_physics_new_initial_state() {
        let s = PhysicsState::new();
        assert_eq!(s.position, [0.0; 3]);
        assert_eq!(s.velocity, [0.0; 3]);
        assert_eq!(s.rotation, [0.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_forward_vector_identity() {
        let s = PhysicsState::new();
        let fwd = s.forward_vector();
        assert!((fwd[0]).abs() < 1e-6);
        assert!((fwd[1]).abs() < 1e-6);
        assert!((fwd[2] - 1.0).abs() < 1e-6, "expected +Z, got {:?}", fwd);
    }

    #[test]
    fn test_quaternion_from_zero_angle() {
        let q = quaternion_from_axis_angle([1.0, 0.0, 0.0], 0.0);
        assert!((q[3] - 1.0).abs() < 1e-6);
        assert_eq!(q[0], 0.0);
    }

    #[test]
    fn test_quaternion_multiply_by_identity() {
        let id = [0.0, 0.0, 0.0, 1.0];
        let b = [0.1, 0.2, 0.3, 0.8];
        let r = quaternion_multiply(&id, &b);
        for i in 0..4 {
            assert!((r[i] - b[i]).abs() < 1e-6);
        }
    }

    #[test]
    fn test_quaternion_multiply_commutes_same_axis() {
        let q = quaternion_from_axis_angle([0.0, 1.0, 0.0], 0.5);
        let r = quaternion_multiply(&q, &q);
        let expected = quaternion_from_axis_angle([0.0, 1.0, 0.0], 1.0);
        for i in 0..4 {
            assert!((r[i] - expected[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn test_update_zero_throttle_no_movement() {
        let mut s = PhysicsState::new();
        let input = ShipInput { throttle: 0.0, yaw: 0.0, pitch: 0.0, roll: 0.0 };
        let stats = dummy_stats();
        s.update(&input, &stats, 1.0);
        assert_eq!(s.position, [0.0; 3]);
    }

    #[test]
    fn test_update_throttle_moves_forward() {
        let mut s = PhysicsState::new();
        let input = ShipInput { throttle: 1.0, yaw: 0.0, pitch: 0.0, roll: 0.0 };
        let stats = dummy_stats();
        s.update(&input, &stats, 1.0);
        assert!(s.velocity[2] > 0.0, "should have forward velocity");
        assert!(s.position[2] > 0.0, "should have forward position");
    }

    #[test]
    fn test_update_clamps_velocity_to_speed_cap() {
        let mut s = PhysicsState::new();
        s.velocity = [0.0, 0.0, 1000.0];
        let input = ShipInput { throttle: 0.0, yaw: 0.0, pitch: 0.0, roll: 0.0 };
        let stats = dummy_stats();
        s.update(&input, &stats, 1.0);
        let speed = s.velocity.iter().map(|v| v.powi(2)).sum::<f32>().sqrt();
        assert!(speed <= 50.0 + 1e-6, "speed {} > 50", speed);
    }

    #[test]
    fn test_update_applies_drag() {
        let mut s = PhysicsState::new();
        s.velocity = [10.0, 0.0, 0.0];
        let input = ShipInput { throttle: 0.0, yaw: 0.0, pitch: 0.0, roll: 0.0 };
        let stats = dummy_stats();
        s.update(&input, &stats, 1.0);
        assert!(s.velocity[0].abs() < 10.0, "drag should reduce velocity");
        assert!(s.velocity[0] > 0.0, "drag should not reverse velocity");
    }

    #[test]
    fn test_dt_zero_produces_no_change() {
        let mut s = PhysicsState::new();
        s.velocity = [10.0, 20.0, 30.0];
        let input = ShipInput { throttle: 1.0, yaw: 0.5, pitch: 0.3, roll: 0.1 };
        let stats = dummy_stats();
        let original = s.clone();
        s.update(&input, &stats, 0.0);
        assert_eq!(s.position, original.position, "position should not change with dt=0");
        assert_eq!(s.velocity, original.velocity, "velocity should not change with dt=0");
        assert_eq!(s.rotation, original.rotation, "rotation should not change with dt=0");
    }

    #[test]
    fn test_negative_dt_clamped() {
        let mut s = PhysicsState::new();
        s.velocity = [10.0, 0.0, 0.0];
        let input = ShipInput { throttle: 0.0, yaw: 0.0, pitch: 0.0, roll: 0.0 };
        let stats = dummy_stats();
        s.update(&input, &stats, -1.0);
        let speed = s.velocity.iter().map(|v| v.powi(2)).sum::<f32>().sqrt();
        assert!(speed.is_finite(), "speed should be finite, got {}", speed);
        assert!(
            speed <= stats.speed + 1e-6,
            "speed {} should not exceed speed_cap {}",
            speed,
            stats.speed,
        );
    }

    #[test]
    fn test_large_dt_numerically_stable() {
        let mut s = PhysicsState::new();
        s.velocity = [10.0, 20.0, 30.0];
        let input = ShipInput { throttle: 1.0, yaw: 0.1, pitch: 0.2, roll: 0.0 };
        let stats = dummy_stats();
        s.update(&input, &stats, 1000.0);
        for c in s.position.iter() {
            assert!(c.is_finite(), "position component should be finite, got {}", c);
        }
        for c in s.velocity.iter() {
            assert!(c.is_finite(), "velocity component should be finite, got {}", c);
        }
        for c in s.rotation.iter() {
            assert!(c.is_finite(), "rotation component should be finite, got {}", c);
        }
    }

    #[test]
    fn test_forward_vector_unit_length_after_rotation() {
        let mut s = PhysicsState::new();
        let input = ShipInput { throttle: 0.0, yaw: 1.0, pitch: 0.5, roll: 0.3 };
        let stats = dummy_stats();
        s.update(&input, &stats, 1.0);
        let fwd = s.forward_vector();
        let mag = (fwd[0].powi(2) + fwd[1].powi(2) + fwd[2].powi(2)).sqrt();
        assert!(
            (mag - 1.0).abs() < 1e_6,
            "forward vector magnitude {}, expected 1.0",
            mag,
        );
    }
}
