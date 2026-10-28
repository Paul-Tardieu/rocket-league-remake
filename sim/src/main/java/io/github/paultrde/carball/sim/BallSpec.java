package io.github.paultrde.carball.sim;

/**
 * @param radius ball radius in units
 * @param mass ball mass, only meaningful relative to {@link CarSpec#mass()}
 * @param restitution fraction of normal speed kept after a bounce, in [0, 1]
 * @param drag fraction of its velocity the ball loses per second to air and rolling friction
 */
public record BallSpec(double radius, double mass, double restitution, double drag) {

    public static final BallSpec DEFAULT = new BallSpec(92, 30, 0.6, 0.2);

    public BallSpec {
        Checks.positive("radius", radius);
        Checks.positive("mass", mass);
        Checks.inRange("restitution", restitution, 0, 1);
        Checks.inRange("drag", drag, 0, 1);
    }
}
