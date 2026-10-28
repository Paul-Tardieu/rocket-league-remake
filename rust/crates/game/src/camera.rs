use crate::render::to_vec3;
use carball_sim::{GameState, PhysicsConfig};
use macroquad::prelude::{Camera3D, Vec3, vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraMode {
    /// Behind the followed car, always looking towards the ball.
    Ball,
    /// Behind the followed car, looking where it drives.
    Car,
    /// High side view of the whole pitch, following the ball like a TV broadcast.
    Broadcast,
}

impl CameraMode {
    pub fn next(self) -> Self {
        match self {
            CameraMode::Ball => CameraMode::Car,
            CameraMode::Car => CameraMode::Broadcast,
            CameraMode::Broadcast => CameraMode::Ball,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CameraMode::Ball => "balle",
            CameraMode::Car => "voiture",
            CameraMode::Broadcast => "tribune",
        }
    }
}

const DISTANCE: f32 = 560.0;
const HEIGHT: f32 = 210.0;
/// How quickly the camera catches up with where it wants to be, in 1/s.
const POSITION_STIFFNESS: f32 = 9.0;
const TARGET_STIFFNESS: f32 = 14.0;

/// A camera that eases towards its desired pose instead of snapping, which hides the jitter of
/// sudden direction changes (flips, bumps) and is much easier on the eyes.
pub struct CameraRig {
    position: Vec3,
    target: Vec3,
    initialized: bool,
}

impl CameraRig {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            target: Vec3::ZERO,
            initialized: false,
        }
    }

    pub fn update(
        &mut self,
        mode: CameraMode,
        state: &GameState,
        config: &PhysicsConfig,
        followed: usize,
        dt: f32,
    ) {
        let (position, target) = desired_pose(mode, state, config, followed);
        if !self.initialized {
            (self.position, self.target, self.initialized) = (position, target, true);
            return;
        }
        self.position = self
            .position
            .lerp(position, 1.0 - (-POSITION_STIFFNESS * dt).exp());
        self.target = self
            .target
            .lerp(target, 1.0 - (-TARGET_STIFFNESS * dt).exp());
    }

    pub fn camera(&self) -> Camera3D {
        Camera3D {
            position: self.position,
            target: self.target,
            up: Vec3::Z,
            fovy: 70.0_f32.to_radians(),
            z_near: 5.0,
            z_far: 40_000.0,
            ..Default::default()
        }
    }
}

fn desired_pose(
    mode: CameraMode,
    state: &GameState,
    config: &PhysicsConfig,
    followed: usize,
) -> (Vec3, Vec3) {
    let ball = to_vec3(state.ball.position);
    let car_state = state.cars.get(followed);
    match (mode, car_state) {
        (CameraMode::Broadcast, _) | (_, None) => {
            let arena = &config.arena;
            let position = vec3(ball.x * 0.7, -(arena.half_width as f32) - 900.0, 1500.0);
            (position, vec3(ball.x * 0.85, ball.y * 0.5, 0.0))
        }
        (CameraMode::Ball, Some(car)) => {
            let car = to_vec3(car.position);
            let to_ball = flat(ball - car).try_normalize().unwrap_or(Vec3::X);
            let position = car - to_ball * DISTANCE + Vec3::Z * HEIGHT;
            (position, car.lerp(ball, 0.65) + Vec3::Z * 40.0)
        }
        (CameraMode::Car, Some(car_state)) => {
            let car = to_vec3(car_state.position);
            let velocity = flat(to_vec3(car_state.velocity));
            let heading = if velocity.length() > 300.0 {
                velocity.normalize()
            } else {
                flat(to_vec3(car_state.forward()))
                    .try_normalize()
                    .unwrap_or(Vec3::X)
            };
            (
                car - heading * DISTANCE + Vec3::Z * HEIGHT,
                car + heading * 500.0 + Vec3::Z * 60.0,
            )
        }
    }
}

fn flat(v: Vec3) -> Vec3 {
    vec3(v.x, v.y, 0.0)
}
