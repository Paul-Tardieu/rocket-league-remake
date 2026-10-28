use crate::arena::confine;
use crate::config::PhysicsConfig;
use crate::state::BallState;
use glam::DVec3;

/// Semi-implicit Euler: update the velocity first, then move with the new velocity.
pub(crate) fn step_ball(ball: &BallState, config: &PhysicsConfig, dt: f64) -> BallState {
    let spec = &config.ball;
    // Linear drag: a fixed fraction of the velocity is lost per second, so fast balls slow down
    // quickly and slow balls drift to a stop without ever reversing direction.
    let mut velocity = cap_speed(ball.velocity, spec.max_speed) * (1.0 - spec.drag * dt)
        + DVec3::new(0.0, 0.0, config.gravity * dt);
    let mut position = ball.position + velocity * dt;

    if position.z < spec.radius {
        position.z = spec.radius;
        let bounce_speed = -velocity.z * spec.restitution;
        // Below this speed a bounce is just gravity accumulated over one tick: resting contact.
        // Without it the ball jitters on the floor forever.
        let resting = bounce_speed < 2.0 * config.gravity.abs() * dt;
        velocity.z = if resting { 0.0 } else { bounce_speed };
    }

    let confined = confine(&config.arena, ball.position, position, spec.radius);
    if confined.hit_end_wall {
        velocity.x *= -spec.restitution;
    }
    if confined.hit_side_wall {
        velocity.y *= -spec.restitution;
    }
    if confined.hit_ceiling {
        velocity.z = -velocity.z.abs() * spec.restitution;
    }
    BallState {
        position: confined.position,
        velocity,
    }
}

/// Car-ball contacts are summed, so a ball squeezed between two cars can be bounced back and forth
/// and gain far more speed than either car had. Capping the speed bounds that energy gain.
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

    fn no_drag() -> PhysicsConfig {
        let mut config = PhysicsConfig::default();
        config.ball.drag = 0.0;
        config
    }

    #[test]
    fn bounces_off_a_side_wall_and_loses_speed() {
        let config = no_drag();
        let limit = config.arena.half_width - config.ball.radius;
        let ball = BallState {
            position: DVec3::new(0.0, limit - 1.0, config.ball.radius),
            velocity: DVec3::new(0.0, 1000.0, 0.0),
        };

        let bounced = step_ball(&ball, &config, config.tick_seconds());

        assert_eq!(bounced.position.y, limit);
        assert_eq!(bounced.velocity.y, -1000.0 * config.ball.restitution);
    }

    #[test]
    fn bounces_off_the_ceiling() {
        let config = no_drag();
        let ceiling = config.arena.ceiling - config.ball.radius;
        let ball = BallState {
            position: DVec3::new(0.0, 0.0, ceiling - 1.0),
            velocity: DVec3::new(0.0, 0.0, 3000.0),
        };

        let bounced = step_ball(&ball, &config, config.tick_seconds());

        assert_eq!(bounced.position.z, ceiling);
        assert!(bounced.velocity.z < 0.0);
    }

    #[test]
    fn caps_the_speed() {
        let config = no_drag();
        let ball = BallState {
            position: DVec3::new(0.0, 0.0, 1000.0),
            velocity: DVec3::new(20_000.0, 0.0, 0.0),
        };

        let capped = step_ball(&ball, &config, config.tick_seconds());

        assert!(capped.velocity.length() <= config.ball.max_speed + 10.0);
    }

    #[test]
    fn drag_slows_a_rolling_ball() {
        let config = PhysicsConfig::default();
        let mut ball = BallState {
            position: DVec3::new(0.0, 0.0, config.ball.radius),
            velocity: DVec3::new(1000.0, 0.0, 0.0),
        };

        for _ in 0..config.ticks_per_second {
            ball = step_ball(&ball, &config, config.tick_seconds());
        }

        // 20 % per second compounded over 120 small ticks: about e^-0.2.
        assert!((ball.velocity.x - 1000.0 * (-0.2f64).exp()).abs() < 2.0);
    }

    #[test]
    fn a_dropped_ball_comes_to_rest() {
        let config = PhysicsConfig::default();
        let mut ball = BallState::at_rest(DVec3::new(0.0, 0.0, 500.0));

        for _ in 0..10 * config.ticks_per_second {
            ball = step_ball(&ball, &config, config.tick_seconds());
            assert!(ball.position.z >= config.ball.radius);
        }

        assert_eq!(ball.velocity, DVec3::ZERO);
    }
}
