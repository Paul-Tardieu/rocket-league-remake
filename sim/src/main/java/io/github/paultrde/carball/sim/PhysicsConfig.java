package io.github.paultrde.carball.sim;

/**
 * Tunable physics constants.
 *
 * @param ticksPerSecond fixed simulation rate; never derived from wall-clock time
 * @param gravity acceleration in units/s², negative is down
 * @param ballRadius ball radius in units
 * @param ballRestitution fraction of normal speed kept after a bounce, in [0, 1]
 */
public record PhysicsConfig(int ticksPerSecond, double gravity, double ballRadius, double ballRestitution) {

    public static final PhysicsConfig DEFAULT = new PhysicsConfig(120, -650, 92, 0.6);

    public PhysicsConfig {
        if (ticksPerSecond <= 0) {
            throw new IllegalArgumentException("ticksPerSecond must be positive: " + ticksPerSecond);
        }
        if (ballRestitution < 0 || ballRestitution > 1) {
            throw new IllegalArgumentException("ballRestitution must be in [0, 1]: " + ballRestitution);
        }
    }

    public double tickSeconds() {
        return 1.0 / ticksPerSecond;
    }
}
