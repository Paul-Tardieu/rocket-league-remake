# Carball

Minimalist, original car-soccer game in Java, built to train bots (reinforcement learning).

## Commands
- `./gradlew build` — compile, test, check formatting
- `./gradlew test` — tests only (`./gradlew :sim:test --tests '*WorldTest'` for one class)
- `./gradlew spotlessApply` — format code (run before committing)
- `./gradlew :render:run` — run the game

## Architecture
- `sim/` — headless deterministic simulation. NO rendering, I/O, threads, wall-clock time or unseeded randomness.
- `render/` — display + input, reads `sim` state only. Depends on `sim`, never the reverse.
- `build-logic/` — shared Gradle conventions (`carball.java-conventions`). Versions live in `gradle/libs.versions.toml`.
- Planned: a Gym-style bridge (`reset` / `step(actions)` → obs, reward, done) so Python can train bots.

## Conventions
- Java 25. Prefer `record` for value/state types, `sealed` interfaces + pattern matching for variants.
- Composition over inheritance; classes `final` unless designed for extension. No mutable `static` state.
- Fixed timestep (`PhysicsConfig.ticksPerSecond`). Units: Z is up, distances in arbitrary units (uu), seconds.
- Validate invariants in record compact constructors.
- Every physics change gets a JUnit 5 + AssertJ test; keep determinism tests green.
- Compiler runs with `-Xlint:all -Werror`: fix warnings, don't suppress them.
- Small focused commits; code, identifiers and comments in English.
