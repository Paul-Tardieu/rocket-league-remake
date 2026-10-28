package io.github.paultrde.carball.sim;

/**
 * Car handling. The car hitbox is a sphere for now, which keeps collisions simple.
 *
 * @param radius hitbox radius in units
 * @param mass car mass, only meaningful relative to {@link BallSpec#mass()}
 * @param acceleration forward acceleration at full throttle, in units/s²
 * @param coastDeceleration deceleration with no throttle, in units/s²
 * @param maxSpeed top ground speed, in units/s
 * @param turnRate yaw rate at full steer and top speed, in rad/s
 * @param jumpSpeed vertical speed given by a jump, in units/s
 */
public record CarSpec(
        double radius,
        double mass,
        double acceleration,
        double coastDeceleration,
        double maxSpeed,
        double turnRate,
        double jumpSpeed) {

    public static final CarSpec DEFAULT = new CarSpec(60, 180, 1600, 500, 2300, 3, 500);

    public CarSpec {
        Checks.positive("radius", radius);
        Checks.positive("mass", mass);
        Checks.positive("acceleration", acceleration);
        Checks.positive("coastDeceleration", coastDeceleration);
        Checks.positive("maxSpeed", maxSpeed);
        Checks.positive("turnRate", turnRate);
        Checks.positive("jumpSpeed", jumpSpeed);
    }
}
