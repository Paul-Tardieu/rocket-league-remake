//! Head-to-head between striker variants: `cargo run --release -p carball-bot --example versus`.

use carball_bot::{Bot, StrikerBot};
use carball_sim::{PhysicsConfig, Team, World};

fn variant(name: &str, car: usize, team: Team, seed: u64) -> StrikerBot {
    let bot = StrikerBot::new(car, team, PhysicsConfig::default(), seed);
    match name {
        "full" => bot,
        "no-boost" => bot.without_boost(),
        "no-flip" => bot.without_flips(),
        "plain" => bot.without_boost().without_flips(),
        _ => panic!("unknown variant {name}"),
    }
}

/// Goals for (a, b) over several 5-minute matches, swapping sides to cancel any side bias.
fn head_to_head(a: &str, b: &str) -> (u32, u32) {
    let (mut goals_a, mut goals_b) = (0, 0);
    for round in 0..6u64 {
        let a_is_blue = round % 2 == 0;
        let (blue, orange) = if a_is_blue { (a, b) } else { (b, a) };
        let mut bots = [
            variant(blue, 0, Team::Blue, 2 * round + 1),
            variant(orange, 1, Team::Orange, 2 * round + 2),
        ];
        let mut world = World::one_versus_one(PhysicsConfig::default());
        for _ in 0..300 * 120 {
            let state = world.state();
            let inputs = [bots[0].act(&state), bots[1].act(&state)];
            world.step(&inputs);
        }
        let score = world.state().score;
        let (blue_goals, orange_goals) = (score.blue, score.orange);
        if a_is_blue {
            goals_a += blue_goals;
            goals_b += orange_goals;
        } else {
            goals_a += orange_goals;
            goals_b += blue_goals;
        }
    }
    (goals_a, goals_b)
}

fn main() {
    for (a, b) in [
        ("full", "plain"),
        ("no-flip", "plain"),
        ("no-boost", "plain"),
        ("full", "no-flip"),
    ] {
        let (ga, gb) = head_to_head(a, b);
        println!("{a:>8} vs {b:<8}: {ga} - {gb}  (6 matches of 5 min)");
    }
}
