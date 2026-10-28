use carball_bot::StrikerBot;
use carball_bot::harness::{Outcome, bot_match, solo_attempt, solo_benchmark};
use carball_sim::{BallState, CarState, DVec3, PhysicsConfig, Team, World};

fn kickoff_world(car_x: f64, car_yaw: f64) -> World {
    let config = PhysicsConfig::default();
    let ball = BallState::at_rest(DVec3::new(0.0, 0.0, config.ball.radius));
    let car = CarState::grounded(
        car_x,
        0.0,
        car_yaw,
        config.car.hitbox_radius,
        config.boost.kickoff_amount,
    );
    World::new(config, ball, vec![car])
}

#[test]
fn scores_from_kickoff() {
    let mut world = kickoff_world(-2000.0, 0.0);
    let mut bot = StrikerBot::new(0, Team::Blue, PhysicsConfig::default(), 1);

    assert_eq!(solo_attempt(&mut world, &mut bot, 30), Outcome::Scored);
}

#[test]
fn orange_attacks_the_other_way() {
    let mut world = kickoff_world(2000.0, std::f64::consts::PI);
    let mut bot = StrikerBot::new(0, Team::Orange, PhysicsConfig::default(), 1);

    // `solo_attempt` reports goals from BLUE's point of view: an ORANGE goal is an "own goal".
    assert_eq!(solo_attempt(&mut world, &mut bot, 30), Outcome::OwnGoal);
}

/// The bot is a heuristic, so single hand-picked cases make a poor test: it fails on some hard
/// spots by design. Instead measure its success rate over many random but reproducible starts.
#[test]
fn scores_from_most_random_starts_without_own_goals() {
    let (scored, own_goals, _) = solo_benchmark(100, 1234);

    assert!(scored >= 75, "scored {scored}/100");
    assert!(own_goals <= 3, "own goals {own_goals}/100");
}

/// Regression test: a squeezed ball used to fly out of the arena, and identical bots mirrored
/// each other forever on a symmetric kickoff.
#[test]
fn bot_versus_bot_keeps_the_ball_inside_and_both_teams_score() {
    let config = PhysicsConfig::default();
    let (arena, r) = (config.arena.clone(), config.ball.radius);
    let tolerance = 1e-6;

    let world = bot_match(300, (1, 2), |state| {
        let ball = state.ball.position;
        assert!(
            ball.y.abs() <= arena.half_width - r + tolerance,
            "ball out sideways: {ball}"
        );
        assert!(
            ball.x.abs() <= arena.half_length + arena.goal_depth - r + tolerance,
            "ball out the back: {ball}"
        );
        assert!(
            ball.z >= r - tolerance && ball.z <= arena.ceiling - r + tolerance,
            "ball out vertically: {ball}"
        );
    });

    let score = world.state().score;
    assert!(score.blue > 0 && score.orange > 0, "score {score:?}");
}
