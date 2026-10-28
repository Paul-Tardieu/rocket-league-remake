package io.github.paultrde.carball.sim;

/**
 * Deterministic fixed-timestep simulation. The same initial state and the same sequence of
 * {@link #step()} calls always produce the same result, which is required for bot training.
 */
public final class World {

    private final PhysicsConfig config;
    private BallState ball;
    private long tick;

    public World(PhysicsConfig config, BallState initialBall) {
        this.config = config;
        this.ball = initialBall;
    }

    public void step() {
        double dt = config.tickSeconds();
        Vec3 velocity = ball.velocity().plus(new Vec3(0, 0, config.gravity() * dt));
        Vec3 position = ball.position().plus(velocity.times(dt));

        if (position.z() < config.ballRadius()) {
            position = new Vec3(position.x(), position.y(), config.ballRadius());
            velocity = new Vec3(velocity.x(), velocity.y(), -velocity.z() * config.ballRestitution());
        }

        ball = new BallState(position, velocity);
        tick++;
    }

    public BallState ball() {
        return ball;
    }

    public long tick() {
        return tick;
    }
}
