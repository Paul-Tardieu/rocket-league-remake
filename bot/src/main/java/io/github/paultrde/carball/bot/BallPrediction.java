package io.github.paultrde.carball.bot;

import io.github.paultrde.carball.sim.BallState;
import io.github.paultrde.carball.sim.PhysicsConfig;
import io.github.paultrde.carball.sim.Vec3;

/**
 * Predicts where an airborne ball will first touch the floor, using projectile motion. It ignores
 * drag, walls and the ceiling, so it is an estimate that gets more accurate as the ball comes down.
 */
final class BallPrediction {

    /**
     * @param position where the ball will be resting on the floor (z = ball radius)
     * @param seconds time until then; 0 if the ball is already on the floor
     */
    record Landing(Vec3 position, double seconds) {}

    private final double gravity;
    private final double ballRadius;

    BallPrediction(PhysicsConfig config) {
        this.gravity = -config.gravity();
        this.ballRadius = config.ball().radius();
    }

    Landing landing(BallState ball) {
        double height = ball.position().z() - ballRadius;
        double verticalSpeed = ball.velocity().z();
        if (height <= 0 && verticalSpeed <= 0) {
            return new Landing(ball.position(), 0);
        }
        // Solve height + vz·t - g·t²/2 = 0 for the positive root.
        double seconds = (verticalSpeed + Math.sqrt(verticalSpeed * verticalSpeed + 2 * gravity * height)) / gravity;
        Vec3 drift = ball.velocity().withZ(0).times(seconds);
        return new Landing(ball.position().plus(drift).withZ(ballRadius), seconds);
    }
}
