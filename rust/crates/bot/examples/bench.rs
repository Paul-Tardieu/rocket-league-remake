//! Measures the striker: `cargo run --release -p carball-bot --example bench`.

use carball_bot::harness::{bot_match, solo_benchmark};

fn main() {
    for seed in [42, 7] {
        let (scored, own, none) = solo_benchmark(200, seed);
        println!("solo seed {seed}: scored {scored}/200, own goals {own}, no goal {none}");
    }

    let mut max_height: f64 = 0.0;
    let mut touches_in_air = 0;
    let world = bot_match(300, (1, 2), |state| {
        max_height = max_height.max(state.ball.position.z);
        touches_in_air += state.cars.iter().filter(|c| !c.on_ground).count();
    });
    let score = world.state().score;
    println!(
        "5 min bot vs bot: blue {} - {} orange, ball max height {max_height:.0}, car-ticks airborne {touches_in_air}",
        score.blue, score.orange
    );
}
