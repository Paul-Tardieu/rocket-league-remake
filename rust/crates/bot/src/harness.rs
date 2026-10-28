//! Headless match helpers shared by the tests and the benchmark example.

use crate::{Bot, SplitMix64, StrikerBot};
use carball_sim::{BallState, CarState, DVec3, GameEvent, GameState, PhysicsConfig, Team, World};

/// How a solo attempt ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Scored,
    OwnGoal,
    NoGoal,
}

/// Runs a lone BLUE striker until the first goal or the time limit.
pub fn solo_attempt(world: &mut World, bot: &mut dyn Bot, max_seconds: u32) -> Outcome {
    let ticks = max_seconds * world.config().ticks_per_second;
    for _ in 0..ticks {
        let input = bot.act(&world.state());
        for event in world.step(&[input]) {
            if let GameEvent::Goal { scorer } = event {
                return if scorer == Team::Blue {
                    Outcome::Scored
                } else {
                    Outcome::OwnGoal
                };
            }
        }
    }
    Outcome::NoGoal
}

/// A lone BLUE car and the ball at random, reproducible spots.
pub fn random_solo_world(rng: &mut SplitMix64) -> World {
    let config = PhysicsConfig::default();
    let mut coordinate = |half: f64, margin: f64| (rng.next_f64() * 2.0 - 1.0) * (half - margin);
    let (hl, hw) = (config.arena.half_length, config.arena.half_width);
    let ball = BallState::at_rest(DVec3::new(
        coordinate(hl, 400.0),
        coordinate(hw, 300.0),
        config.ball.radius,
    ));
    let (x, y) = (coordinate(hl, 400.0), coordinate(hw, 300.0));
    let yaw = coordinate(std::f64::consts::PI, 0.0);
    let car = CarState::grounded(
        x,
        y,
        yaw,
        config.car.hitbox_radius,
        config.boost.kickoff_amount,
    );
    World::new(config, ball, vec![car])
}

/// Success counts of a lone striker over `runs` random starts.
pub fn solo_benchmark(runs: u32, seed: u64) -> (u32, u32, u32) {
    let mut rng = SplitMix64::new(seed);
    let (mut scored, mut own, mut none) = (0, 0, 0);
    for run in 0..runs {
        let mut world = random_solo_world(&mut rng);
        let mut bot = StrikerBot::new(0, Team::Blue, world.config().clone(), u64::from(run));
        match solo_attempt(&mut world, &mut bot, 30) {
            Outcome::Scored => scored += 1,
            Outcome::OwnGoal => own += 1,
            Outcome::NoGoal => none += 1,
        }
    }
    (scored, own, none)
}

/// Plays a 1v1 between two strikers and calls `observe` after every tick.
pub fn bot_match(seconds: u32, seeds: (u64, u64), mut observe: impl FnMut(&GameState)) -> World {
    let config = PhysicsConfig::default();
    let mut world = World::one_versus_one(config.clone());
    let mut bots = [
        StrikerBot::new(0, Team::Blue, config.clone(), seeds.0),
        StrikerBot::new(1, Team::Orange, config.clone(), seeds.1),
    ];
    for _ in 0..seconds * config.ticks_per_second {
        let state = world.state();
        let inputs = [bots[0].act(&state), bots[1].act(&state)];
        world.step(&inputs);
        observe(&world.state());
    }
    world
}
