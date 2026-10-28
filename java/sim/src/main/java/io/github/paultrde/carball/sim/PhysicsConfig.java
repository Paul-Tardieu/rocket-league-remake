package io.github.paultrde.carball.sim;

import java.util.Objects;

/**
 * Tunable physics constants.
 *
 * @param ticksPerSecond fixed simulation rate; never derived from wall-clock time
 * @param gravity acceleration in units/s², negative is down
 */
public record PhysicsConfig(int ticksPerSecond, double gravity, ArenaSpec arena, BallSpec ball, CarSpec car) {

    public static final PhysicsConfig DEFAULT =
            new PhysicsConfig(120, -650, ArenaSpec.DEFAULT, BallSpec.DEFAULT, CarSpec.DEFAULT);

    public PhysicsConfig {
        if (ticksPerSecond <= 0) {
            throw new IllegalArgumentException("ticksPerSecond must be positive: " + ticksPerSecond);
        }
        Objects.requireNonNull(arena, "arena");
        Objects.requireNonNull(ball, "ball");
        Objects.requireNonNull(car, "car");
    }

    public double tickSeconds() {
        return 1.0 / ticksPerSecond;
    }
}
