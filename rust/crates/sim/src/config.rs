//! Tunable constants. Distances are in arbitrary units (uu), times in seconds, Z is up.

use glam::DVec3;

/// Rectangular arena centred on the origin: floor at z = 0, flat ceiling, a goal in each end wall.
#[derive(Clone, Debug, PartialEq)]
pub struct ArenaSpec {
    /// Distance from the centre to each goal line, along X.
    pub half_length: f64,
    /// Distance from the centre to each side wall, along Y.
    pub half_width: f64,
    pub goal_half_width: f64,
    pub goal_height: f64,
    /// Depth of the net behind the goal line.
    pub goal_depth: f64,
    pub ceiling: f64,
}

impl ArenaSpec {
    pub const DEFAULT: Self = Self {
        half_length: 4000.0,
        half_width: 3000.0,
        goal_half_width: 900.0,
        goal_height: 640.0,
        goal_depth: 880.0,
        ceiling: 2000.0,
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct BallSpec {
    pub radius: f64,
    /// Only meaningful relative to [`CarSpec::mass`].
    pub mass: f64,
    /// Fraction of the normal speed kept after a bounce, in [0, 1].
    pub restitution: f64,
    /// Fraction of its velocity the ball loses per second to air and rolling friction.
    pub drag: f64,
    /// Speed cap, so no pile-up of collisions can launch the ball absurdly fast.
    pub max_speed: f64,
}

impl BallSpec {
    pub const DEFAULT: Self = Self {
        radius: 92.0,
        mass: 30.0,
        restitution: 0.6,
        drag: 0.2,
        max_speed: 6000.0,
    };
}

/// Car handling. The collision hitbox is a sphere; the box shape is only drawn by the renderer.
#[derive(Clone, Debug, PartialEq)]
pub struct CarSpec {
    pub hitbox_radius: f64,
    pub mass: f64,
    /// Ground acceleration at full throttle, up to [`Self::max_drive_speed`].
    pub throttle_accel: f64,
    /// Deceleration when the throttle opposes the current direction of travel.
    pub brake_accel: f64,
    /// Deceleration with no throttle.
    pub coast_decel: f64,
    /// Top speed reachable with the throttle alone.
    pub max_drive_speed: f64,
    /// Absolute top speed of the car, reachable with boost.
    pub max_speed: f64,
    /// Yaw rate at full steer, in rad/s.
    pub turn_rate: f64,
    /// Speed from which the full turn rate is available; slower cars turn proportionally less.
    pub full_turn_speed: f64,
    pub jump_speed: f64,
    pub double_jump_speed: f64,
    /// Time after the first jump during which a second jump (flip or double jump) is allowed.
    pub flip_window: f64,
    /// Horizontal speed added by a flip in the flip direction.
    pub dodge_speed: f64,
    /// Duration of one full flip rotation.
    pub dodge_duration: f64,
    /// Fraction of gravity applied while flipping, so a low flip completes its turn before landing.
    pub dodge_gravity_scale: f64,
    pub boost_accel: f64,
    /// Boost consumed per second while boosting, out of 100.
    pub boost_per_second: f64,
    /// Air control rates at full input, in rad/s.
    pub air_pitch_rate: f64,
    pub air_yaw_rate: f64,
    pub air_roll_rate: f64,
    /// How quickly the car's spin follows the air-control input, in 1/s.
    pub air_response: f64,
}

impl CarSpec {
    pub const DEFAULT: Self = Self {
        hitbox_radius: 60.0,
        mass: 180.0,
        throttle_accel: 1600.0,
        brake_accel: 3500.0,
        coast_decel: 500.0,
        max_drive_speed: 1400.0,
        max_speed: 2300.0,
        turn_rate: 3.0,
        full_turn_speed: 600.0,
        jump_speed: 550.0,
        double_jump_speed: 450.0,
        flip_window: 1.25,
        dodge_speed: 700.0,
        dodge_duration: 0.55,
        dodge_gravity_scale: 0.3,
        boost_accel: 1050.0,
        boost_per_second: 33.0,
        air_pitch_rate: 5.0,
        air_yaw_rate: 3.5,
        air_roll_rate: 5.5,
        air_response: 12.0,
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoostSpec {
    /// Boost every car has at kickoff, out of 100.
    pub kickoff_amount: f64,
    pub small_amount: f64,
    pub big_amount: f64,
    pub small_respawn: f64,
    pub big_respawn: f64,
    /// Horizontal distance from a pad's centre within which a grounded car collects it.
    pub small_pickup_radius: f64,
    pub big_pickup_radius: f64,
}

impl BoostSpec {
    pub const DEFAULT: Self = Self {
        kickoff_amount: 33.0,
        small_amount: 12.0,
        big_amount: 100.0,
        small_respawn: 4.0,
        big_respawn: 10.0,
        small_pickup_radius: 120.0,
        big_pickup_radius: 170.0,
    };
}

/// A boost pad lying on the floor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoostPad {
    pub position: DVec3,
    pub big: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PhysicsConfig {
    /// Fixed simulation rate; never derived from wall-clock time.
    pub ticks_per_second: u32,
    /// Vertical acceleration, negative is down.
    pub gravity: f64,
    pub arena: ArenaSpec,
    pub ball: BallSpec,
    pub car: CarSpec,
    pub boost: BoostSpec,
    pub pads: Vec<BoostPad>,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            ticks_per_second: 120,
            gravity: -650.0,
            arena: ArenaSpec::DEFAULT,
            ball: BallSpec::DEFAULT,
            car: CarSpec::DEFAULT,
            boost: BoostSpec::DEFAULT,
            pads: default_pads(),
        }
    }
}

impl PhysicsConfig {
    pub fn tick_seconds(&self) -> f64 {
        1.0 / f64::from(self.ticks_per_second)
    }
}

/// Pad layout, symmetric under both mirrors so neither team is favoured.
fn default_pads() -> Vec<BoostPad> {
    let big = [(3400.0, 2500.0), (0.0, 2650.0)];
    let small = [
        (1800.0, 0.0),
        (2600.0, 1300.0),
        (1000.0, 1700.0),
        (0.0, 1100.0),
    ];
    let mut pads = Vec::new();
    for (positions, is_big) in [(&big[..], true), (&small[..], false)] {
        for &(x, y) in positions {
            let mut mirrored = vec![(x, y), (-x, y), (x, -y), (-x, -y)];
            mirrored.sort_by(|a, b| a.partial_cmp(b).expect("pad coordinates are finite"));
            mirrored.dedup();
            pads.extend(mirrored.into_iter().map(|(x, y)| BoostPad {
                position: DVec3::new(x, y, 0.0),
                big: is_big,
            }));
        }
    }
    pads
}
