# Carball

Minimalist, original car-soccer game built to train bots (reinforcement learning).
Two implementations live side by side:

- `rust/` — **active version**: 3D view, boost, flips. All new work goes here.
- `java/` — first prototype (2D top-down Swing view). Frozen; only touch it if asked.

## Commands (Rust, run from `rust/`)
- `cargo test` — all tests (sim + bot)
- `cargo fmt` then `cargo clippy --all-targets -- -D warnings` — before committing
- `cargo run --release -p game` — play vs the bot; add `-- --watch` (bot vs bot), `--speed=4`,
  `--screenshot=out.png --after=3` (save a frame after 3 sim seconds, then exit; used to check visuals)

Java (from `java/`): `./gradlew build`, `./gradlew :render:run`.

## Rust architecture (`rust/crates/`)
- `sim` — headless deterministic physics. Only dependency: `glam`. NO rendering, I/O, threads,
  wall-clock time or unseeded randomness. `World::step(&inputs) -> Vec<GameEvent>`, `World::state()`.
  Z is up, units ≈ cm-like "uu", seconds. Fixed 120 ticks/s.
- `bot` — `Bot` trait (`&GameState -> CarInput`), `StrikerBot` hand-written baseline. Its tests check
  a success rate over seeded random starts, not single cases.
- `game` — macroquad 3D window + keyboard. Depends on `bot` → `sim`, never the reverse.

## Conventions
- Physics changes need a test; keep the determinism and fairness tests green (car order must not matter).
- Car-ball contacts are resolved against the same ball and summed (no first-car bias).
- Bots get their randomness from an explicit seed.
- Code, identifiers and comments in English; user-facing replies in French.
- The user judges results, not code: verify visuals with `--screenshot` and behaviour with headless runs.
