use carball_sim::{BallState, DVec3, PhysicsConfig};

/// Where and when an airborne ball first touches the floor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Landing {
    /// Ball resting on the floor (z = ball radius).
    pub position: DVec3,
    /// Time until then; 0 if the ball is already on the floor.
    pub seconds: f64,
}

/// Projectile motion only: ignores drag, walls and the ceiling, so it is an estimate that gets more
/// accurate as the ball comes down.
pub fn landing(ball: &BallState, config: &PhysicsConfig) -> Landing {
    let gravity = -config.gravity;
    let radius = config.ball.radius;
    let height = ball.position.z - radius;
    let vertical_speed = ball.velocity.z;
    if height <= 0.0 && vertical_speed <= 0.0 {
        return Landing {
            position: ball.position,
            seconds: 0.0,
        };
    }
    // Solve height + vz·t - g·t²/2 = 0 for the positive root.
    let seconds = (vertical_speed
        + (vertical_speed * vertical_speed + 2.0 * gravity * height).sqrt())
        / gravity;
    let mut position = ball.position + ball.velocity * seconds;
    position.z = radius;
    Landing { position, seconds }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_resting_ball_lands_where_it_is() {
        let config = PhysicsConfig::default();
        let ball = BallState::at_rest(DVec3::new(100.0, 200.0, config.ball.radius));

        assert_eq!(
            landing(&ball, &config),
            Landing {
                position: ball.position,
                seconds: 0.0
            }
        );
    }

    #[test]
    fn a_lob_lands_further_along_its_path() {
        let config = PhysicsConfig::default();
        let ball = BallState {
            position: DVec3::new(0.0, 0.0, 500.0),
            velocity: DVec3::new(1000.0, 0.0, 300.0),
        };

        let predicted = landing(&ball, &config);

        assert!(predicted.seconds > 1.0);
        assert!((predicted.position.x - 1000.0 * predicted.seconds).abs() < 1e-9);
        assert_eq!(predicted.position.z, config.ball.radius);
    }
}
