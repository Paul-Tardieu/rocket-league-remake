package io.github.paultrde.carball.sim;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.within;

import org.junit.jupiter.api.Test;

class BallPhysicsTest {

    private static final BallSpec NO_DRAG = new BallSpec(92, 30, 0.6, 0);
    private static final PhysicsConfig CONFIG =
            new PhysicsConfig(120, -650, ArenaSpec.DEFAULT, NO_DRAG, CarSpec.DEFAULT);
    private static final double DT = CONFIG.tickSeconds();

    @Test
    void bouncesOffSideWallAndLosesSpeed() {
        BallPhysics physics = new BallPhysics(CONFIG);
        double wallLimit = CONFIG.arena().halfWidth() - NO_DRAG.radius();
        BallState nearWall = new BallState(new Vec3(0, wallLimit - 1, NO_DRAG.radius()), new Vec3(0, 1000, 0));

        BallState bounced = physics.step(nearWall, DT);

        assertThat(bounced.position().y()).isEqualTo(wallLimit);
        assertThat(bounced.velocity().y()).isEqualTo(-1000 * NO_DRAG.restitution());
    }

    @Test
    void dragSlowsARollingBall() {
        BallPhysics physics = new BallPhysics(PhysicsConfig.DEFAULT);
        BallState ball = new BallState(new Vec3(0, 0, NO_DRAG.radius()), new Vec3(1000, 0, 0));

        for (int i = 0; i < PhysicsConfig.DEFAULT.ticksPerSecond(); i++) {
            ball = physics.step(ball, DT);
        }

        // Losing 20 % per second, compounded over 120 small ticks: about e^-0.2 ≈ 0.82.
        assertThat(ball.velocity().x()).isCloseTo(1000 * Math.exp(-0.2), within(2.0));
    }
}
