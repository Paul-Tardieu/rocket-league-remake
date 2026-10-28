package io.github.paultrde.carball.sim;

/**
 * @param radius ball radius in units
 * @param mass ball mass, only meaningful relative to {@link CarSpec#mass()}
 * @param restitution fraction of normal speed kept after a bounce, in [0, 1]
 */
public record BallSpec(double radius, double mass, double restitution) {

    public static final BallSpec DEFAULT = new BallSpec(92, 30, 0.6);

    public BallSpec {
        Checks.positive("radius", radius);
        Checks.positive("mass", mass);
        Checks.inRange("restitution", restitution, 0, 1);
    }
}
