use crate::ball::step_ball;
use crate::car::step_car;
use crate::config::PhysicsConfig;
use crate::contact::{Body, sphere_contact};
use crate::state::{BallState, CarInput, CarState, GameEvent, GameState, Score, Team};
use glam::DVec3;

/// Restitution of car-vs-car bumps: soft, so cars do not ping-pong off each other.
const CAR_BUMP_RESTITUTION: f64 = 0.3;

/// Deterministic fixed-timestep simulation of a match: the same kickoff and the same inputs always
/// produce the same result, which is required for bot training.
#[derive(Clone, Debug)]
pub struct World {
    config: PhysicsConfig,
    kickoff_ball: BallState,
    kickoff_cars: Vec<CarState>,
    ball: BallState,
    cars: Vec<CarState>,
    pad_respawn: Vec<f64>,
    score: Score,
    tick: u64,
}

impl World {
    /// Starts a match; the given positions are also used for every kickoff after a goal.
    pub fn new(
        config: PhysicsConfig,
        kickoff_ball: BallState,
        kickoff_cars: Vec<CarState>,
    ) -> Self {
        let pad_respawn = vec![0.0; config.pads.len()];
        Self {
            ball: kickoff_ball,
            cars: kickoff_cars.clone(),
            config,
            kickoff_ball,
            kickoff_cars,
            pad_respawn,
            score: Score::default(),
            tick: 0,
        }
    }

    /// A 1v1 match: car 0 is BLUE, car 1 is ORANGE, on point-symmetric diagonal kickoff spots.
    pub fn one_versus_one(config: PhysicsConfig) -> Self {
        let r = config.car.hitbox_radius;
        let boost = config.boost.kickoff_amount;
        let (x, y) = (2200.0_f64, 1200.0_f64);
        let blue_yaw = y.atan2(x);
        let cars = vec![
            CarState::grounded(-x, -y, blue_yaw, r, boost),
            CarState::grounded(x, y, blue_yaw + std::f64::consts::PI, r, boost),
        ];
        let ball = BallState::at_rest(DVec3::new(0.0, 0.0, config.ball.radius));
        Self::new(config, ball, cars)
    }

    pub fn config(&self) -> &PhysicsConfig {
        &self.config
    }

    pub fn state(&self) -> GameState {
        GameState {
            ball: self.ball,
            cars: self.cars.clone(),
            pad_respawn: self.pad_respawn.clone(),
            score: self.score,
            tick: self.tick,
        }
    }

    /// Advances the simulation by one tick and returns what happened, in order.
    ///
    /// # Panics
    /// If `inputs` does not hold exactly one input per car.
    pub fn step(&mut self, inputs: &[CarInput]) -> Vec<GameEvent> {
        assert_eq!(inputs.len(), self.cars.len(), "expected one input per car");
        let dt = self.config.tick_seconds();
        let mut events = Vec::new();

        let ball = step_ball(&self.ball, &self.config, dt);
        let mut cars: Vec<CarState> = self
            .cars
            .iter()
            .zip(inputs)
            .map(|(car, input)| step_car(car, &input.sanitized(), &self.config, dt))
            .collect();
        self.ball = self.resolve_contacts(ball, &mut cars, &mut events);
        self.collect_boost(&mut cars, &mut events, dt);
        self.cars = cars;
        self.tick += 1;

        if self.is_goal() {
            let scorer = Team::scoring_on_side(self.ball.position.x);
            self.score = self.score.with_goal_for(scorer);
            events.push(GameEvent::Goal { scorer });
            self.place_for_kickoff();
        }
        events
    }

    /// Starts a fresh match: kickoff positions, full pads, score 0-0, tick 0.
    pub fn reset(&mut self) {
        self.place_for_kickoff();
        self.score = Score::default();
        self.tick = 0;
    }

    /// Every contact is computed from the same pre-contact states and the results are summed.
    /// Applying contacts one after another would let car 0 always act first, a bias bots could
    /// learn to exploit.
    fn resolve_contacts(
        &self,
        ball: BallState,
        cars: &mut [CarState],
        events: &mut Vec<GameEvent>,
    ) -> BallState {
        let ball_body = Body {
            position: ball.position,
            velocity: ball.velocity,
            radius: self.config.ball.radius,
            mass: self.config.ball.mass,
        };
        let car_body = |car: &CarState| Body {
            position: car.position,
            velocity: car.velocity,
            radius: self.config.car.hitbox_radius,
            mass: self.config.car.mass,
        };

        let (mut ball_push, mut ball_kick) = (DVec3::ZERO, DVec3::ZERO);
        let mut car_push = vec![DVec3::ZERO; cars.len()];
        let mut car_kick = vec![DVec3::ZERO; cars.len()];

        for (i, car) in cars.iter().enumerate() {
            if let Some(contact) =
                sphere_contact(car_body(car), ball_body, self.config.ball.restitution)
            {
                events.push(GameEvent::BallTouch { car: i });
                car_push[i] += contact.push_a;
                car_kick[i] += contact.kick_a;
                ball_push += contact.push_b;
                ball_kick += contact.kick_b;
            }
        }
        for i in 0..cars.len() {
            for j in i + 1..cars.len() {
                if let Some(contact) =
                    sphere_contact(car_body(&cars[i]), car_body(&cars[j]), CAR_BUMP_RESTITUTION)
                {
                    car_push[i] += contact.push_a;
                    car_kick[i] += contact.kick_a;
                    car_push[j] += contact.push_b;
                    car_kick[j] += contact.kick_b;
                }
            }
        }

        for (i, car) in cars.iter_mut().enumerate() {
            car.position += car_push[i];
            car.velocity += car_kick[i];
        }
        BallState {
            position: ball.position + ball_push,
            velocity: ball.velocity + ball_kick,
        }
    }

    /// Cars on the floor collect available pads they drive over. Every car on a pad during the same
    /// tick gets it, so the car order never decides who wins a pad.
    fn collect_boost(&mut self, cars: &mut [CarState], events: &mut Vec<GameEvent>, dt: f64) {
        let spec = &self.config.boost;
        for (pad, respawn) in self.config.pads.iter().zip(self.pad_respawn.iter_mut()) {
            *respawn = (*respawn - dt).max(0.0);
            if *respawn > 0.0 {
                continue;
            }
            let (radius, amount, cooldown) = if pad.big {
                (spec.big_pickup_radius, spec.big_amount, spec.big_respawn)
            } else {
                (
                    spec.small_pickup_radius,
                    spec.small_amount,
                    spec.small_respawn,
                )
            };
            let mut taken = false;
            for (i, car) in cars.iter_mut().enumerate() {
                let near = (car.position - pad.position).truncate().length() <= radius;
                if near && car.on_ground && car.boost < 100.0 {
                    car.boost = (car.boost + amount).min(100.0);
                    events.push(GameEvent::BoostPickup {
                        car: i,
                        big: pad.big,
                    });
                    taken = true;
                }
            }
            if taken {
                *respawn = cooldown;
            }
        }
    }

    /// A goal counts once the whole ball is past the goal line.
    fn is_goal(&self) -> bool {
        self.ball.position.x.abs() > self.config.arena.half_length + self.config.ball.radius
    }

    fn place_for_kickoff(&mut self) {
        self.ball = self.kickoff_ball;
        self.cars.clone_from(&self.kickoff_cars);
        self.pad_respawn.fill(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn throttle(value: f64) -> CarInput {
        CarInput {
            throttle: value,
            ..CarInput::default()
        }
    }

    fn lone_car_world(ball: BallState) -> World {
        let config = PhysicsConfig::default();
        let car = CarState::grounded(-1000.0, 0.0, 0.0, config.car.hitbox_radius, 0.0);
        World::new(config, ball, vec![car])
    }

    #[test]
    fn same_inputs_give_the_same_match() {
        let mut first = World::one_versus_one(PhysicsConfig::default());
        let mut second = first.clone();

        for i in 0..2000 {
            let wobble = (f64::from(i) * 0.01).sin();
            let inputs = [
                CarInput {
                    throttle: 1.0,
                    steer: wobble,
                    jump: i % 200 < 5,
                    boost: i % 3 == 0,
                    pitch: 1.0,
                    ..CarInput::default()
                },
                CarInput {
                    throttle: 1.0,
                    steer: -wobble,
                    ..CarInput::default()
                },
            ];
            first.step(&inputs);
            second.step(&inputs);
        }

        assert_eq!(first.state(), second.state());
    }

    #[test]
    fn car_order_does_not_matter_on_a_head_on_kickoff() {
        let config = PhysicsConfig::default();
        let r = config.car.hitbox_radius;
        let ball = BallState::at_rest(DVec3::new(0.0, 0.0, config.ball.radius));
        let cars = vec![
            CarState::grounded(-2500.0, 0.0, 0.0, r, 0.0),
            CarState::grounded(2500.0, 0.0, std::f64::consts::PI, r, 0.0),
        ];
        let ceiling = config.arena.ceiling - config.ball.radius;
        let mut world = World::new(config, ball, cars);

        for _ in 0..600 {
            world.step(&[throttle(1.0), throttle(1.0)]);
            assert!(world.state().ball.position.z <= ceiling);
        }

        // The squeezed ball goes straight up: neither car gets an edge from being first.
        assert!(world.state().ball.position.x.abs() < 1e-6);
    }

    #[test]
    fn driving_into_the_ball_pushes_it_and_reports_the_touch() {
        let mut world = lone_car_world(BallState::at_rest(DVec3::new(0.0, 0.0, 92.0)));

        let touched = (0..240).any(|_| {
            world
                .step(&[throttle(1.0)])
                .contains(&GameEvent::BallTouch { car: 0 })
        });

        assert!(touched);
        assert!(world.state().ball.position.x > 0.0);
    }

    #[test]
    fn a_ball_over_the_line_scores_and_resets_for_kickoff() {
        let config = PhysicsConfig::default();
        let about_to_score = BallState {
            position: DVec3::new(config.arena.half_length, 0.0, 200.0),
            velocity: DVec3::new(3000.0, 0.0, 0.0),
        };
        let mut world = lone_car_world(about_to_score);
        let kickoff_car = world.state().cars[0];

        let goal = (0..120).find_map(|_| {
            world
                .step(&[CarInput::default()])
                .into_iter()
                .find(|e| matches!(e, GameEvent::Goal { .. }))
        });

        assert_eq!(goal, Some(GameEvent::Goal { scorer: Team::Blue }));
        assert_eq!(world.state().score, Score { blue: 1, orange: 0 });
        assert_eq!(world.state().cars[0], kickoff_car);
    }

    #[test]
    fn reset_clears_score_and_tick() {
        let mut world = World::one_versus_one(PhysicsConfig::default());
        world.score = Score { blue: 3, orange: 2 };
        world.step(&[CarInput::default(), CarInput::default()]);

        world.reset();

        assert_eq!(world.state().score, Score::default());
        assert_eq!(world.state().tick, 0);
    }

    #[test]
    fn a_car_collects_a_pad_which_then_respawns() {
        let config = PhysicsConfig::default();
        let pad_index = config
            .pads
            .iter()
            .position(|p| !p.big)
            .expect("a small pad");
        let pad = config.pads[pad_index];
        let car = CarState::grounded(
            pad.position.x,
            pad.position.y,
            0.0,
            config.car.hitbox_radius,
            0.0,
        );
        let ball = BallState::at_rest(DVec3::new(0.0, 0.0, 92.0));
        let respawn = config.boost.small_respawn;
        let ticks_per_second = f64::from(config.ticks_per_second);
        let mut world = World::new(config, ball, vec![car]);

        let events = world.step(&[CarInput::default()]);

        assert!(events.contains(&GameEvent::BoostPickup { car: 0, big: false }));
        assert_eq!(world.state().cars[0].boost, 12.0);
        assert!(world.state().pad_respawn[pad_index] > 0.0);
        // Still parked on the pad: nothing until it respawns, then it is collected again.
        // (±2 ticks of slack for floating-point rounding of the countdown.)
        for _ in 0..(respawn * ticks_per_second) as u32 - 2 {
            world.step(&[CarInput::default()]);
        }
        assert_eq!(world.state().cars[0].boost, 12.0);
        for _ in 0..4 {
            world.step(&[CarInput::default()]);
        }
        assert_eq!(world.state().cars[0].boost, 24.0);
    }

    #[test]
    fn cars_bump_instead_of_passing_through_each_other() {
        let config = PhysicsConfig::default();
        let r = config.car.hitbox_radius;
        let cars = vec![
            CarState::grounded(-500.0, 2000.0, 0.0, r, 0.0),
            CarState::grounded(500.0, 2000.0, std::f64::consts::PI, r, 0.0),
        ];
        let mut world = World::new(config, BallState::at_rest(DVec3::new(0.0, 0.0, 92.0)), cars);

        for _ in 0..240 {
            world.step(&[throttle(1.0), throttle(1.0)]);
            let state = world.state();
            let gap = (state.cars[1].position - state.cars[0].position).length();
            assert!(gap > 2.0 * r - 30.0, "cars overlap: gap {gap}");
        }
    }

    #[test]
    #[should_panic(expected = "one input per car")]
    fn rejects_the_wrong_number_of_inputs() {
        World::one_versus_one(PhysicsConfig::default()).step(&[]);
    }
}
