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
            *component -= *component * drag * dt;
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
