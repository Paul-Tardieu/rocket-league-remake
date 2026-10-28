package io.github.paultrde.carball.sim;

final class BallPhysics {

    private final BallSpec spec;
    private final double gravity;

    BallPhysics(BallSpec spec, double gravity) {
        this.spec = spec;
        this.gravity = gravity;
    }

    /** Semi-implicit Euler: update velocity first, then move with the new velocity. */
    BallState step(BallState ball, double dt) {
        Vec3 velocity = ball.velocity().plus(new Vec3(0, 0, gravity * dt));
        Vec3 position = ball.position().plus(velocity.times(dt));

        if (position.z() < spec.radius()) {
            position = position.withZ(spec.radius());
            double bounceSpeed = -velocity.z() * spec.restitution();
            // Below this speed a bounce is just gravity accumulated over one tick: treat it as resting
            // contact, otherwise the ball jitters on the floor forever.
            boolean resting = bounceSpeed < 2 * Math.abs(gravity) * dt;
            velocity = velocity.withZ(resting ? 0 : bounceSpeed);
        }
        return new BallState(position, velocity);
    }
}
