//! Plain data describing a match. Everything here is cheap to clone and safe to hand to bots.

use glam::{DQuat, DVec3};

/// What a player or bot asks its car to do during one tick: the action space of a bot.
///
/// Analog values are in [-1, 1]; the world clamps them (and turns NaN into 0) before use.
/// On the ground only `throttle`, `steer`, `jump` and `boost` matter. In the air `pitch`, `yaw` and
/// `roll` spin the car, and a second jump becomes a flip in the direction of (`pitch`, `yaw`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CarInput {
    /// 1 full forward, -1 full reverse / brake.
    pub throttle: f64,
    /// 1 turns left, -1 turns right.
    pub steer: f64,
    /// 1 tips the nose down (like pushing a stick forward), -1 lifts it.
    pub pitch: f64,
    /// 1 turns the nose left, -1 right.
    pub yaw: f64,
    /// 1 rolls right, -1 rolls left.
    pub roll: f64,
    /// Held state; the car reacts when it goes from released to pressed.
    pub jump: bool,
    pub boost: bool,
}

impl CarInput {
    pub(crate) fn sanitized(self) -> Self {
        let axis = |v: f64| if v.is_nan() { 0.0 } else { v.clamp(-1.0, 1.0) };
        Self {
            throttle: axis(self.throttle),
            steer: axis(self.steer),
            pitch: axis(self.pitch),
            yaw: axis(self.yaw),
            roll: axis(self.roll),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BallState {
    pub position: DVec3,
    pub velocity: DVec3,
}

impl BallState {
    pub fn at_rest(position: DVec3) -> Self {
        Self {
            position,
            velocity: DVec3::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarState {
    pub position: DVec3,
    pub velocity: DVec3,
    /// Rotation from car space (X forward, Y left, Z up) to world space.
    pub orientation: DQuat,
    /// World-space angular velocity, in rad/s.
    pub angular_velocity: DVec3,
    /// Boost left, out of 100.
    pub boost: f64,
    pub on_ground: bool,
    /// True during a tick where the car burned boost; used by renderers for the flame.
    pub boosting: bool,
    /// Whether the second jump (flip or double jump) is still available.
    pub can_flip: bool,
    pub seconds_since_jump: f64,
    /// Time left in the current flip rotation; air control is disabled meanwhile.
    pub flip_seconds_left: f64,
    /// Jump input of the previous tick, to detect presses.
    pub jump_held: bool,
}

impl CarState {
    /// A car standing on the floor at `(x, y)`, facing `yaw` radians (0 = +X, counter-clockwise).
    pub fn grounded(x: f64, y: f64, yaw: f64, hitbox_radius: f64, boost: f64) -> Self {
        Self {
            position: DVec3::new(x, y, hitbox_radius),
            velocity: DVec3::ZERO,
            orientation: DQuat::from_rotation_z(yaw),
            angular_velocity: DVec3::ZERO,
            boost,
            on_ground: true,
            boosting: false,
            can_flip: false,
            seconds_since_jump: 0.0,
            flip_seconds_left: 0.0,
            jump_held: false,
        }
    }

    pub fn forward(&self) -> DVec3 {
        self.orientation * DVec3::X
    }

    pub fn left(&self) -> DVec3 {
        self.orientation * DVec3::Y
    }

    pub fn up(&self) -> DVec3 {
        self.orientation * DVec3::Z
    }

    /// Heading in radians, 0 = +X, counter-clockwise seen from above.
    pub fn yaw(&self) -> f64 {
        let forward = self.forward();
        if forward.truncate().length() > 0.3 {
            forward.y.atan2(forward.x)
        } else {
            // Nose pointing almost straight up or down: read the heading from the left axis.
            let left = self.left();
            (-left.x).atan2(left.y)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Team {
    /// Defends the goal at -X, attacks +X.
    Blue,
    /// Defends the goal at +X, attacks -X.
    Orange,
}

impl Team {
    /// +1 if this team scores in the goal at +X, -1 otherwise.
    pub fn attack_sign(self) -> f64 {
        match self {
            Team::Blue => 1.0,
            Team::Orange => -1.0,
        }
    }

    /// The team that scores when the ball ends up on this side of the field.
    pub fn scoring_on_side(x: f64) -> Team {
        if x > 0.0 { Team::Blue } else { Team::Orange }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Score {
    pub blue: u32,
    pub orange: u32,
}

impl Score {
    pub fn with_goal_for(self, team: Team) -> Score {
        match team {
            Team::Blue => Score {
                blue: self.blue + 1,
                ..self
            },
            Team::Orange => Score {
                orange: self.orange + 1,
                ..self
            },
        }
    }
}

/// Something notable that happened during a tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameEvent {
    /// The ball fully crossed a goal line; positions have already been reset for kickoff.
    Goal {
        scorer: Team,
    },
    /// A car hit the ball. Useful as a reward signal for bots.
    BallTouch {
        car: usize,
    },
    BoostPickup {
        car: usize,
        big: bool,
    },
}

/// Immutable snapshot of a match.
#[derive(Clone, Debug, PartialEq)]
pub struct GameState {
    pub ball: BallState,
    /// Cars in a stable order; a car's index is its identity.
    pub cars: Vec<CarState>,
    /// Seconds until each pad of [`crate::PhysicsConfig::pads`] is available again; 0 = available.
    pub pad_respawn: Vec<f64>,
    pub score: Score,
    pub tick: u64,
}
