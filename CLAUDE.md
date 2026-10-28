# Carball

Minimalist, original car-soccer game built to train bots (reinforcement learning).
Two implementations live side by side:

- `training/` — **the real goal**: train bots for actual Rocket League with RLGym v2 + RocketSim +
  RLGym-PPO (Python 3.12 via uv). `uv run python smoke_test.py`, `uv run python train.py` (needs a real TTY).
  `training/collision_meshes/` holds meshes dumped from the user's own game: never commit or share them.
- `rust/` — Carball, our own original car-soccer game (3D view, boost, flips). A playground, not a
  training target: its physics deliberately differ from Rocket League.
- `java/` — first prototype (2D top-down Swing view). Frozen; only touch it if asked.

## Training status (2026-10-29)
- A 30M-step run with the current `train.py` rewards learned nothing: entropy stayed ~4.495 (max ln 90 ≈ 4.4998),
  the policy is near-uniform and touches the ball no more than random play.
- Diagnosis: reward mix. Next step is the beginner rewards from the RLGym-PPO guide
  (https://github.com/ZealanL/RLGym-PPO-Guide/blob/main/making_a_good_bot.md): touch 50, speed toward ball 5,
  face ball 1, in air 0.15, and NO goal / ball-to-goal rewards until the bot touches the ball; 50k timesteps per
  iteration, lr 2e-4. Validate with a ~15 min run (entropy dropping, touches well above random via
  `replay.py`) before any long run.
- Training will move to the user's Windows desktop (has Rocket League): copy `training/collision_meshes/`
  (or re-dump it there); rlgym-ppo resumes from `training/data/checkpoints/` automatically. On Windows, PyTorch
  from PyPI is CPU-only: install the CUDA build from the PyTorch index.

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
