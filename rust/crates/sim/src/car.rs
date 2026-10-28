//! Arcade car model.
//!
//! On the ground the tyres have perfect grip: the car moves along its heading only. In the air the
//! car keeps its momentum, can spin with pitch/yaw/roll, boost along its nose, and use one second
//! jump within [`CarSpec::flip_window`]: a flip (dodge) if a direction is held, else a double jump.
//! Landing always puts the car back on its wheels, which keeps the game forgiving for bots.

use crate::arena::confine;
use crate::config::{CarSpec, PhysicsConfig};
use crate::state::{CarInput, CarState};
use glam::{DQuat, DVec2, DVec3};
use std::f64::consts::TAU;

/// Below this stick deflection a second jump is a double jump rather than a flip.
const FLIP_DEADZONE: f64 = 0.3;

pub(crate) fn step_car(
    car: &CarState,
    input: &CarInput,
    config: &PhysicsConfig,
    dt: f64,
) -> CarState {
    let spec = &config.car;
    let mut car = *car;
    let jump_pressed = input.jump && !car.jump_held;
    car.jump_held = input.jump;
    car.boosting = input.boost && car.boost > 0.0;

    if car.on_ground {
        drive(&mut car, input, spec, dt);
        if jump_pressed {
            jump(&mut car, spec);
        }
    } else {
        fly(&mut car, input, spec, jump_pressed, dt);
    }

    if car.boosting {
        car.boost = (car.boost - spec.boost_per_second * dt).max(0.0);
        if !car.on_ground {
            car.velocity += car.forward() * spec.boost_accel * dt;
        }
    }
    if !car.on_ground {
        let gravity_scale = if car.flip_seconds_left > 0.0 {
            spec.dodge_gravity_scale
        } else {
            1.0
        };
        car.velocity.z += config.gravity * gravity_scale * dt;
    }
    car.velocity = cap_speed(car.velocity, spec.max_speed);

    let previous = car.position;
    car.position += car.velocity * dt;
    if !car.on_ground {
        advance_rotation(&mut car, spec, dt);
    }
    resolve_floor(&mut car, spec);

    let confined = confine(&config.arena, previous, car.position, spec.hitbox_radius);
    car.position = confined.position;
    if confined.hit_end_wall {
        car.velocity.x = 0.0;
    }
    if confined.hit_side_wall {
        car.velocity.y = 0.0;
    }
    if confined.hit_ceiling {
        car.velocity.z = car.velocity.z.min(0.0);
    }
    car
}

fn drive(car: &mut CarState, input: &CarInput, spec: &CarSpec, dt: f64) {
    let yaw = car.yaw();
    let mut speed = car.velocity.dot(heading(yaw));

    let throttle = input.throttle;
    if throttle != 0.0 {
        if speed * throttle < 0.0 {
            speed += throttle * spec.brake_accel * dt;
        } else if speed.abs() < spec.max_drive_speed {
            // The throttle alone tops out at max_drive_speed; only boost goes beyond.
            speed = (speed + throttle * spec.throttle_accel * dt)
                .clamp(-spec.max_drive_speed, spec.max_drive_speed);
        }
    } else {
        // Coasting slows the car towards zero without overshooting into reverse.
        speed = speed.signum() * (speed.abs() - spec.coast_decel * dt).max(0.0);
    }
    if car.boosting {
        speed += spec.boost_accel * dt;
    }
    speed = speed.clamp(-spec.max_drive_speed, spec.max_speed);

    // Yaw rate grows with speed up to full_turn_speed, then stays capped: tight turns when slow,
    // wide turns when fast. No rotation at rest, and steering is mirrored in reverse.
    let yaw_rate = input.steer * spec.turn_rate * (speed / spec.full_turn_speed).clamp(-1.0, 1.0);
    let new_yaw = yaw + yaw_rate * dt;
    car.orientation = DQuat::from_rotation_z(new_yaw);
    car.velocity = heading(new_yaw) * speed;
    car.angular_velocity = DVec3::new(0.0, 0.0, yaw_rate);
}

fn jump(car: &mut CarState, spec: &CarSpec) {
    car.velocity += car.up() * spec.jump_speed;
    car.on_ground = false;
    car.can_flip = true;
    car.seconds_since_jump = 0.0;
}

fn fly(car: &mut CarState, input: &CarInput, spec: &CarSpec, jump_pressed: bool, dt: f64) {
    if jump_pressed && car.can_flip && car.seconds_since_jump <= spec.flip_window {
        let stick = DVec2::new(input.pitch, input.yaw);
        if stick.length() > FLIP_DEADZONE {
            dodge(car, stick.normalize(), spec);
        } else {
            car.velocity += car.up() * spec.double_jump_speed;
        }
        car.can_flip = false;
    }

    if car.flip_seconds_left <= 0.0 {
        // Air control drives the spin towards a target rate rather than applying raw torque:
        // easier to steer for humans and bots alike.
        let target_local = DVec3::new(
            input.roll * spec.air_roll_rate,
            input.pitch * spec.air_pitch_rate,
            input.yaw * spec.air_yaw_rate,
        );
        let target = car.orientation * target_local;
        let blend = (spec.air_response * dt).min(1.0);
        car.angular_velocity += (target - car.angular_velocity) * blend;
    }
}

/// A flip: a burst of horizontal speed in the stick direction plus one full tumble that way.
///
/// `direction` is (forward, left) relative to the car's heading.
fn dodge(car: &mut CarState, direction: DVec2, spec: &CarSpec) {
    let yaw = car.yaw();
    let forward = heading(yaw);
    let left = DVec3::new(-yaw.sin(), yaw.cos(), 0.0);
    let world_direction = forward * direction.x + left * direction.y;

    car.velocity += world_direction * spec.dodge_speed;
    // Cancelling the vertical speed keeps a flip low and snappy: the car does not keep floating
    // up from the jump, so it is back on its wheels quickly.
    car.velocity.z = 0.0;
    // Rotating about (up × direction) tips the nose towards the flip direction:
    // a front flip spins about the car's left axis, nose down.
    let axis = DVec3::Z.cross(world_direction).normalize();
    car.angular_velocity = axis * (TAU / spec.dodge_duration);
    car.flip_seconds_left = spec.dodge_duration;
}

fn advance_rotation(car: &mut CarState, spec: &CarSpec, dt: f64) {
    car.orientation =
        (DQuat::from_scaled_axis(car.angular_velocity * dt) * car.orientation).normalize();
    car.seconds_since_jump += dt;
    if car.seconds_since_jump > spec.flip_window {
        car.can_flip = false;
    }
    if car.flip_seconds_left > 0.0 {
        car.flip_seconds_left -= dt;
        if car.flip_seconds_left <= 1e-9 {
            // The flip is exactly one turn long; stop spinning so the car ends level.
            car.flip_seconds_left = 0.0;
            car.angular_velocity = DVec3::ZERO;
        }
    }
}

fn resolve_floor(car: &mut CarState, spec: &CarSpec) {
    let ride_height = spec.hitbox_radius;
    if car.position.z <= ride_height {
        car.position.z = ride_height;
        if !car.on_ground && car.velocity.z <= 0.0 {
            land(car);
        }
    } else if car.on_ground {
        // Knocked upwards by a collision: airborne, but without a jump to flip from.
        car.on_ground = false;
        car.can_flip = false;
    }
}

/// Puts the car back on its wheels, keeping its heading.
fn land(car: &mut CarState) {
    car.orientation = DQuat::from_rotation_z(car.yaw());
    car.velocity.z = 0.0;
    car.angular_velocity = DVec3::ZERO;
    car.on_ground = true;
    car.can_flip = false;
    car.flip_seconds_left = 0.0;
}

fn heading(yaw: f64) -> DVec3 {
    DVec3::new(yaw.cos(), yaw.sin(), 0.0)
}

fn cap_speed(velocity: DVec3, max_speed: f64) -> DVec3 {
    let speed = velocity.length();
    if speed > max_speed {
        velocity * (max_speed / speed)
    } else {
        velocity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> PhysicsConfig {
        PhysicsConfig::default()
    }

    fn parked(config: &PhysicsConfig, boost: f64) -> CarState {
        CarState::grounded(0.0, 0.0, 0.0, config.car.hitbox_radius, boost)
    }

    fn run(car: CarState, input: CarInput, seconds: f64, config: &PhysicsConfig) -> CarState {
        let ticks = (seconds * f64::from(config.ticks_per_second)).round() as u32;
        (0..ticks).fold(car, |car, _| {
            step_car(&car, &input, config, config.tick_seconds())
        })
    }

    fn full_throttle() -> CarInput {
        CarInput {
            throttle: 1.0,
            ..CarInput::default()
        }
    }

    #[test]
    fn throttle_alone_tops_out_at_max_drive_speed() {
        let config = config();
        let car = run(parked(&config, 0.0), full_throttle(), 2.0, &config);

        assert!((car.velocity.length() - config.car.max_drive_speed).abs() < 1e-6);
    }

    #[test]
    fn boost_reaches_but_never_exceeds_max_speed() {
        let config = config();
        let boosting = CarInput {
            boost: true,
            ..full_throttle()
        };
        let car = run(parked(&config, 100.0), boosting, 2.4, &config);

        assert!((car.velocity.length() - config.car.max_speed).abs() < 1e-6);
    }

    #[test]
    fn boosting_burns_the_tank_then_stops() {
        let config = config();
        let boosting = CarInput {
            boost: true,
            ..CarInput::default()
        };

        let half = run(parked(&config, 100.0), boosting, 1.0, &config);
        assert!((half.boost - (100.0 - config.car.boost_per_second)).abs() < 1e-6);

        let empty = run(half, boosting, 5.0, &config);
        assert_eq!(empty.boost, 0.0);
        assert!(!empty.boosting);
    }

    #[test]
    fn cannot_turn_while_stopped() {
        let config = config();
        let car = run(
            parked(&config, 0.0),
            CarInput {
                steer: 1.0,
                ..CarInput::default()
            },
            1.0,
            &config,
        );

        assert_eq!(car.yaw(), 0.0);
    }

    #[test]
    fn jumps_only_from_the_ground_and_lands_back_upright() {
        let config = config();
        let jump = CarInput {
            jump: true,
            ..CarInput::default()
        };
        let airborne = step_car(&parked(&config, 0.0), &jump, &config, config.tick_seconds());
        assert!(!airborne.on_ground);
        assert!(airborne.can_flip);

        let landed = run(
            airborne,
            CarInput {
                pitch: 1.0,
                ..CarInput::default()
            },
            3.0,
            &config,
        );

        assert!(landed.on_ground);
        assert!((landed.up() - DVec3::Z).length() < 1e-9);
    }

    #[test]
    fn double_jump_goes_higher_than_a_single_jump() {
        let config = config();
        let dt = config.tick_seconds();
        let jump = CarInput {
            jump: true,
            ..CarInput::default()
        };
        let release = CarInput::default();

        let first = step_car(&parked(&config, 0.0), &jump, &config, dt);
        let released = run(first, release, 0.1, &config);
        let single_peak = peak_height(released, release, &config);
        let doubled = step_car(&released, &jump, &config, dt);
        let double_peak = peak_height(doubled, release, &config);

        assert!(double_peak > single_peak + 100.0);
        assert!(!doubled.can_flip);
    }

    #[test]
    fn front_flip_adds_forward_speed_and_spins_one_full_turn() {
        let config = config();
        let dt = config.tick_seconds();
        let jump = CarInput {
            jump: true,
            ..CarInput::default()
        };
        let first = step_car(&parked(&config, 0.0), &jump, &config, dt);
        let released = run(first, CarInput::default(), 0.1, &config);

        let flip = CarInput {
            jump: true,
            pitch: 1.0,
            ..CarInput::default()
        };
        let flipping = step_car(&released, &flip, &config, dt);
        assert!(flipping.velocity.x > config.car.dodge_speed - 1.0);
        assert!(
            flipping.velocity.z <= 0.0,
            "a flip cancels the upward speed of the jump"
        );

        let halfway = run(
            flipping,
            CarInput::default(),
            config.car.dodge_duration / 2.0,
            &config,
        );
        assert!(
            halfway.forward().x < -0.9,
            "upside down halfway through the flip"
        );

        let done = run(
            halfway,
            CarInput::default(),
            config.car.dodge_duration / 2.0,
            &config,
        );
        assert!(done.forward().x > 0.99, "level again after one full turn");
        assert_eq!(done.angular_velocity, DVec3::ZERO);
    }

    #[test]
    fn no_flip_after_the_flip_window() {
        let config = config();
        let dt = config.tick_seconds();
        let jump = CarInput {
            jump: true,
            ..CarInput::default()
        };
        let first = step_car(&parked(&config, 0.0), &jump, &config, dt);
        // A long double-jump-free hang time: boost upwards to stay airborne past the window.
        let mut car = first;
        car.velocity.z = 2000.0;
        let late = run(
            car,
            CarInput::default(),
            config.car.flip_window + 0.1,
            &config,
        );

        let flip = step_car(
            &late,
            &CarInput {
                jump: true,
                pitch: 1.0,
                ..CarInput::default()
            },
            &config,
            dt,
        );

        assert_eq!(flip.flip_seconds_left, 0.0);
    }

    #[test]
    fn drives_into_the_goal_and_stops_at_the_back_of_the_net() {
        let config = config();
        let car = run(parked(&config, 0.0), full_throttle(), 10.0, &config);

        let arena = &config.arena;
        assert_eq!(
            car.position.x,
            arena.half_length + arena.goal_depth - config.car.hitbox_radius
        );
    }

    #[test]
    fn stops_at_the_end_wall_beside_the_goal() {
        let config = config();
        let beside_goal = CarState::grounded(0.0, 2000.0, 0.0, config.car.hitbox_radius, 0.0);
        let car = run(beside_goal, full_throttle(), 10.0, &config);

        assert_eq!(
            car.position.x,
            config.arena.half_length - config.car.hitbox_radius
        );
    }

    fn peak_height(mut car: CarState, input: CarInput, config: &PhysicsConfig) -> f64 {
        let mut peak = car.position.z;
        while !car.on_ground {
            car = step_car(&car, &input, config, config.tick_seconds());
            peak = peak.max(car.position.z);
        }
        peak
    }
}
