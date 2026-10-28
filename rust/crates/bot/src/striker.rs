use crate::Bot;
use crate::prediction::landing;
use crate::rng::SplitMix64;
use carball_sim::{CarInput, CarState, DVec3, GameState, PhysicsConfig, Team};
use std::f64::consts::PI;

/// Aim offset behind the ball, as a fraction of the car-to-ball distance.
const OFFSET_RATIO: f64 = 0.5;
/// Cap on the aim offset.
const MAX_OFFSET: f64 = 1500.0;
/// Distance behind and beside the ball used to swing around it from the wrong side.
const DETOUR_DISTANCE: f64 = 900.0;
/// How far in front of the ball (towards its target goal) the car may be before detouring.
const GOAL_SIDE_TOLERANCE: f64 = 100.0;
/// Converts a heading error in radians into a steer value.
const STEER_GAIN: f64 = 5.0;
/// Heading error above which the bot brakes to turn tighter.
const SHARP_TURN: f64 = PI / 4.0;
/// Heading error above which, at low speed, reversing turns the car round faster.
const TARGET_BEHIND: f64 = 110.0 * PI / 180.0;
/// Backwards speed above which the car counts as reversing.
const REVERSE_SPEED_EPSILON: f64 = 50.0;
/// Speed the bot brakes down to for sharp turns; slow means a small turning radius.
const CORNERING_SPEED: f64 = 1000.0;
/// Keeps aim points away from the walls so the car does not grind along them.
const WALL_MARGIN: f64 = 250.0;
/// Below this time to landing the ball is playable now; no need to pace the approach.
const PLAYABLE_SECONDS: f64 = 0.3;
/// Standard deviation of the steering noise added every tick.
const STEER_NOISE: f64 = 0.05;
/// Boost only when roughly facing the aim and still this far from it.
const BOOST_MIN_DISTANCE: f64 = 1200.0;
const BOOST_MAX_ERROR: f64 = 0.3;
/// Flip into the ball when it is this close (centre to centre), low, ahead and lined up with goal.
const FLIP_DISTANCE: f64 = 520.0;
const FLIP_MAX_BALL_HEIGHT: f64 = 220.0;
const FLIP_MIN_SPEED: f64 = 1000.0;
const FLIP_MAX_ERROR: f64 = 0.25;
/// No flip if another car is this close to the ball.
const FLIP_CONTEST_DISTANCE: f64 = 1200.0;
const FLIP_LINED_UP_COS: f64 = 0.9;
/// Ticks the jump is held, then released, before the flip input.
const FLIP_JUMP_TICKS: u32 = 3;
const FLIP_WAIT_TICKS: u32 = 3;

/// Multi-tick flip manoeuvre: jump, release, then jump again with the stick pointing at the ball.
#[derive(Clone, Copy, Debug, PartialEq)]
enum FlipPlan {
    Idle,
    Jumping { ticks_left: u32 },
    Waiting { ticks_left: u32 },
    Flipping,
}

/// Hand-written baseline: get behind the ball, then drive through it towards the opponent goal.
///
/// The trick is the aim point. Driving straight at the ball pushes it wherever the car comes from,
/// so the bot aims at a point behind the ball on the goal-to-ball line, and that point slides onto
/// the ball as the car gets closer. From the wrong side of the ball it swings wide around it first.
/// An airborne ball is played where it will land, arriving just in time instead of circling under
/// it. It boosts on straights and finishes with a front flip when lined up for a shot.
///
/// A little seeded noise is added to the steering: two identical deterministic bots facing a
/// symmetric kickoff would otherwise mirror each other forever.
pub struct StrikerBot {
    car_index: usize,
    config: PhysicsConfig,
    target_goal: DVec3,
    rng: SplitMix64,
    flip: FlipPlan,
    uses_boost: bool,
    uses_flips: bool,
}

impl StrikerBot {
    pub fn new(car_index: usize, team: Team, config: PhysicsConfig, seed: u64) -> Self {
        let target_goal = DVec3::new(team.attack_sign() * config.arena.half_length, 0.0, 0.0);
        Self {
            car_index,
            config,
            target_goal,
            rng: SplitMix64::new(seed),
            flip: FlipPlan::Idle,
            uses_boost: true,
            uses_flips: true,
        }
    }

    /// Variant that never boosts: a weaker baseline, useful to measure what boost is worth.
    pub fn without_boost(self) -> Self {
        Self {
            uses_boost: false,
            ..self
        }
    }

    /// Variant that never flips.
    pub fn without_flips(self) -> Self {
        Self {
            uses_flips: false,
            ..self
        }
    }

    fn continue_flip(&mut self, car: &CarState, ball: DVec3) -> Option<CarInput> {
        let hold = CarInput {
            throttle: 1.0,
            ..CarInput::default()
        };
        match self.flip {
            FlipPlan::Idle => None,
            FlipPlan::Jumping { ticks_left } => {
                self.flip = if ticks_left > 1 {
                    FlipPlan::Jumping {
                        ticks_left: ticks_left - 1,
                    }
                } else {
                    FlipPlan::Waiting {
                        ticks_left: FLIP_WAIT_TICKS,
                    }
                };
                Some(CarInput { jump: true, ..hold })
            }
            FlipPlan::Waiting { ticks_left } => {
                self.flip = if ticks_left > 1 {
                    FlipPlan::Waiting {
                        ticks_left: ticks_left - 1,
                    }
                } else {
                    FlipPlan::Flipping
                };
                Some(hold)
            }
            FlipPlan::Flipping => {
                self.flip = FlipPlan::Idle;
                let error = heading_error(car, ball);
                Some(CarInput {
                    jump: true,
                    pitch: 1.0,
                    yaw: (error * 2.0).clamp(-1.0, 1.0),
                    ..hold
                })
            }
        }
    }

    /// A flip commits the car for half a second, so only take shots nobody can contest: with an
    /// opponent close to the ball, the ball would often bounce straight back past the flipping car.
    fn should_flip(&self, state: &GameState, car: &CarState, forward_speed: f64) -> bool {
        let ball = state.ball.position;
        let contested = state.cars.iter().enumerate().any(|(i, other)| {
            i != self.car_index && (other.position - ball).length() < FLIP_CONTEST_DISTANCE
        });
        let to_ball = ball - car.position;
        let ball_to_goal = flat(self.target_goal - ball).normalize_or_zero();
        !contested
            && car.on_ground
            && to_ball.length() < FLIP_DISTANCE
            && ball.z < FLIP_MAX_BALL_HEIGHT
            && forward_speed > FLIP_MIN_SPEED
            && heading_error(car, ball).abs() < FLIP_MAX_ERROR
            && flat(to_ball).normalize_or_zero().dot(ball_to_goal) > FLIP_LINED_UP_COS
    }

    fn choose_aim(&self, car: DVec3, ball: DVec3) -> DVec3 {
        let ball_to_goal = flat(self.target_goal - ball).normalize_or_zero();
        let ball_to_car = flat(car - ball);

        // Aim behind the ball, further behind the further away the car is. The aim slides onto the
        // ball as the car closes in, bending the path into a curve that arrives lined up with goal.
        let offset = (ball_to_car.length() * OFFSET_RATIO).min(MAX_OFFSET);
        let mut aim = flat(ball) - ball_to_goal * offset;

        if ball_to_car.dot(ball_to_goal) > -GOAL_SIDE_TOLERANCE {
            // Car is on the goal side: swing wide around the ball on whichever side the car is.
            let side = DVec3::new(-ball_to_goal.y, ball_to_goal.x, 0.0);
            let side_sign = if ball_to_car.dot(side) >= 0.0 {
                1.0
            } else {
                -1.0
            };
            aim =
                flat(ball) - ball_to_goal * DETOUR_DISTANCE + side * (side_sign * DETOUR_DISTANCE);
        }
        self.keep_off_walls(aim)
    }

    fn keep_off_walls(&self, point: DVec3) -> DVec3 {
        let max_x = self.config.arena.half_length - WALL_MARGIN;
        let max_y = self.config.arena.half_width - WALL_MARGIN;
        DVec3::new(
            point.x.clamp(-max_x, max_x),
            point.y.clamp(-max_y, max_y),
            0.0,
        )
    }
}

impl Bot for StrikerBot {
    fn act(&mut self, state: &GameState) -> CarInput {
        let car = state.cars[self.car_index];
        let ball = state.ball.position;
        if let Some(input) = self.continue_flip(&car, ball) {
            return input;
        }
        if !car.on_ground {
            // Nothing clever in the air: hold the throttle so the car drives off on landing.
            return CarInput {
                throttle: 1.0,
                ..CarInput::default()
            };
        }

        let forward_speed = car.velocity.dot(car.forward());
        if self.uses_flips && self.should_flip(state, &car, forward_speed) {
            self.flip = FlipPlan::Jumping {
                ticks_left: FLIP_JUMP_TICKS,
            };
            return self.continue_flip(&car, ball).expect("flip just started");
        }

        let predicted = landing(&state.ball, &self.config);
        let aim = self.choose_aim(car.position, predicted.position);
        let mut input = drive_towards(&car, aim, forward_speed);
        input.steer = (input.steer + self.rng.gaussian() * STEER_NOISE).clamp(-1.0, 1.0);

        let distance = flat(aim - car.position).length();
        input.boost = self.uses_boost
            && input.throttle > 0.0
            && heading_error(&car, aim).abs() < BOOST_MAX_ERROR
            && distance > BOOST_MIN_DISTANCE;
        if predicted.seconds > PLAYABLE_SECONDS {
            input = pace_arrival(input, distance, predicted.seconds, forward_speed);
        }
        input
    }
}

fn drive_towards(car: &CarState, aim: DVec3, forward_speed: f64) -> CarInput {
    let error = heading_error(car, aim);
    // Steering is mirrored whenever the car actually rolls backwards, whatever the throttle.
    let steer_for = (error * STEER_GAIN).clamp(-1.0, 1.0);
    let steer = if forward_speed < 0.0 {
        -steer_for
    } else {
        steer_for
    };

    // Backing up with the aim behind is a three-point turn; it also frees a car stuck nose-first
    // against a wall. Hysteresis: start reversing only when the aim is well behind, but once rolling
    // backwards keep going until the nose roughly points at the aim. With a single threshold the
    // bot would flip between forward and reverse every tick.
    let rolling_backwards = forward_speed < -REVERSE_SPEED_EPSILON;
    let reverse = if rolling_backwards {
        error.abs() > SHARP_TURN
    } else {
        error.abs() > TARGET_BEHIND && forward_speed < CORNERING_SPEED
    };
    let too_fast_to_turn = error.abs() > SHARP_TURN && forward_speed > CORNERING_SPEED;
    let throttle = if reverse || too_fast_to_turn {
        -1.0
    } else {
        1.0
    };
    CarInput {
        throttle,
        steer,
        ..CarInput::default()
    }
}

/// Arriving early under a falling ball is useless: the car overshoots and loops back. Cap the
/// speed at what reaches the aim just as the ball lands, coasting or braking if faster.
fn pace_arrival(
    input: CarInput,
    distance: f64,
    seconds_to_landing: f64,
    forward_speed: f64,
) -> CarInput {
    if input.throttle <= 0.0 {
        return input;
    }
    let needed_speed = distance / seconds_to_landing;
    if forward_speed <= needed_speed {
        return input;
    }
    let throttle = if forward_speed > needed_speed + CORNERING_SPEED {
        -1.0
    } else {
        0.0
    };
    CarInput {
        throttle,
        boost: false,
        ..input
    }
}

/// Signed angle from the car's heading to the target, in [-π, π].
fn heading_error(car: &CarState, target: DVec3) -> f64 {
    let wanted = (target.y - car.position.y).atan2(target.x - car.position.x);
    wrap_angle(wanted - car.yaw())
}

fn wrap_angle(radians: f64) -> f64 {
    radians.sin().atan2(radians.cos())
}

fn flat(v: DVec3) -> DVec3 {
    DVec3::new(v.x, v.y, 0.0)
}
