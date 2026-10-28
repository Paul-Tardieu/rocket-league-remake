package io.github.paultrde.carball.sim;

import static org.assertj.core.api.Assertions.assertThat;

import org.junit.jupiter.api.Test;

class BallPhysicsTest {

    private static final PhysicsConfig CONFIG = PhysicsConfig.DEFAULT;
    private static final double DT = CONFIG.tickSeconds();

    private final BallPhysics physics = new BallPhysics(CONFIG);

    @Test
    void bouncesOffSideWallAndLosesSpeed() {
        double wallLimit = CONFIG.arena().halfWidth() - CONFIG.ball().radius();
        BallState nearWall =
                new BallState(new Vec3(0, wallLimit - 1, CONFIG.ball().radius()), new Vec3(0, 1000, 0));

        BallState bounced = physics.step(nearWall, DT);

        assertThat(bounced.position().y()).isEqualTo(wallLimit);
        assertThat(bounced.velocity().y()).isEqualTo(-1000 * CONFIG.ball().restitution());
    }
}
