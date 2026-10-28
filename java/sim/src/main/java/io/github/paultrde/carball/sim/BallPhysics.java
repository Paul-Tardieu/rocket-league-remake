package io.github.paultrde.carball.sim;

final class BallPhysics {

    private final BallSpec spec;
    private final ArenaBounds bounds;
    private final double gravity;

    BallPhysics(PhysicsConfig config) {
        this.spec = config.ball();
        this.bounds = new ArenaBounds(config.arena());
        this.gravity = config.gravity();
    }

    /** Semi-implicit Euler: update velocity first, then move with the new velocity. */
    BallState step(BallState ball, double dt) {
        // Linear drag: losing a fixed fraction per second makes fast balls slow down quickly and
        // slow balls drift to a stop, without ever reversing direction.
        Vec3 velocity = capSpeed(ball.velocity()).times(1 - spec.drag() * dt).plus(new Vec3(0, 0, gravity * dt));
        Vec3 position = ball.position().plus(velocity.times(dt));

        if (position.z() < spec.radius()) {
            position = position.withZ(spec.radius());
            double bounceSpeed = -velocity.z() * spec.restitution();
            // Below this speed a bounce is just gravity accumulated over one tick: treat it as resting
            // contact, otherwise the ball jitters on the floor forever.
            boolean resting = bounceSpeed < 2 * Math.abs(gravity) * dt;
            velocity = velocity.withZ(resting ? 0 : bounceSpeed);
        }

        ArenaBounds.Confined confined = bounds.confine(ball.position(), position, spec.radius());
        double bounce = -spec.restitution();
        if (confined.hitEndWall()) {
            velocity = velocity.withX(velocity.x() * bounce);
        }
        if (confined.hitSideWall()) {
            velocity = velocity.withY(velocity.y() * bounce);
        }
        if (confined.hitCeiling()) {
            velocity = velocity.withZ(velocity.z() * bounce);
        }
        return new BallState(confined.position(), velocity);
    }

    /**
     * Collisions are resolved one car at a time, so a ball squeezed between two cars can be bounced
     * back and forth within a few ticks and pick up far more speed than either car had. Capping the
     * speed bounds that energy gain; it is also a gameplay knob.
     */
    private Vec3 capSpeed(Vec3 velocity) {
        double speed = velocity.length();
        return speed > spec.maxSpeed() ? velocity.times(spec.maxSpeed() / speed) : velocity;
    }
}
