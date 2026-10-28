package io.github.paultrde.carball.sim;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatIllegalArgumentException;

import java.util.List;
import org.junit.jupiter.api.Test;

class WorldTest {

    private static final PhysicsConfig CONFIG = PhysicsConfig.DEFAULT;
    private static final BallState DROPPED_BALL = new BallState(new Vec3(0, 0, 500), Vec3.ZERO);
    private static final CarState CAR_FACING_BALL =
            new CarState(new Vec3(-1000, 0, CONFIG.car().radius()), Vec3.ZERO, 0);

    @Test
    void ballNeverSinksBelowTheFloor() {
        World world = new World(CONFIG, DROPPED_BALL, List.of());

        for (int i = 0; i < 10 * CONFIG.ticksPerSecond(); i++) {
            world.step(List.of());
            assertThat(world.ball().position().z())
                    .isGreaterThanOrEqualTo(CONFIG.ball().radius());
        }
    }

    @Test
    void droppedBallEventuallyComesToRest() {
        World world = new World(CONFIG, DROPPED_BALL, List.of());

        for (int i = 0; i < 10 * CONFIG.ticksPerSecond(); i++) {
            world.step(List.of());
        }

        assertThat(world.ball().velocity()).isEqualTo(Vec3.ZERO);
    }

    @Test
    void sameInputsGiveSameState() {
        World first = new World(CONFIG, DROPPED_BALL, List.of(CAR_FACING_BALL));
        World second = new World(CONFIG, DROPPED_BALL, List.of(CAR_FACING_BALL));

        for (int i = 0; i < 1_000; i++) {
            List<CarInput> inputs = List.of(new CarInput(1, Math.sin(i * 0.01), i % 200 == 0));
            first.step(inputs);
            second.step(inputs);
        }

        assertThat(first.ball()).isEqualTo(second.ball());
        assertThat(first.cars()).isEqualTo(second.cars());
        assertThat(first.tick()).isEqualTo(1_000);
    }

    @Test
    void carDrivingIntoBallPushesIt() {
        BallState restingBall = new BallState(new Vec3(0, 0, CONFIG.ball().radius()), Vec3.ZERO);
        World world = new World(CONFIG, restingBall, List.of(CAR_FACING_BALL));

        for (int i = 0; i < 2 * CONFIG.ticksPerSecond(); i++) {
            world.step(List.of(new CarInput(1, 0, false)));
        }

        assertThat(world.ball().position().x()).isPositive();
    }

    @Test
    void rejectsWrongNumberOfInputs() {
        World world = new World(CONFIG, DROPPED_BALL, List.of(CAR_FACING_BALL));

        assertThatIllegalArgumentException().isThrownBy(() -> world.step(List.of()));
    }
}
