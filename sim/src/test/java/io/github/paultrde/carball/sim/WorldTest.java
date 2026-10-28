package io.github.paultrde.carball.sim;

import static org.assertj.core.api.Assertions.assertThat;

import org.junit.jupiter.api.Test;

class WorldTest {

    private static final BallState DROPPED_BALL = new BallState(new Vec3(0, 0, 500), Vec3.ZERO);

    @Test
    void ballNeverSinksBelowTheFloor() {
        World world = new World(PhysicsConfig.DEFAULT, DROPPED_BALL);

        for (int i = 0; i < 10 * PhysicsConfig.DEFAULT.ticksPerSecond(); i++) {
            world.step();
            assertThat(world.ball().position().z()).isGreaterThanOrEqualTo(PhysicsConfig.DEFAULT.ballRadius());
        }
    }

    @Test
    void sameInputsGiveSameState() {
        World first = new World(PhysicsConfig.DEFAULT, DROPPED_BALL);
        World second = new World(PhysicsConfig.DEFAULT, DROPPED_BALL);

        for (int i = 0; i < 1_000; i++) {
            first.step();
            second.step();
        }

        assertThat(first.ball()).isEqualTo(second.ball());
        assertThat(first.tick()).isEqualTo(1_000);
    }
}
