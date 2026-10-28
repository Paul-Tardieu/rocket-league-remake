package io.github.paultrde.carball.sim;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatIllegalArgumentException;
import static org.assertj.core.api.Assertions.within;

import java.util.List;
import org.junit.jupiter.api.Test;

class WorldTest {

    private static final PhysicsConfig CONFIG = PhysicsConfig.DEFAULT;
    private static final ArenaSpec ARENA = CONFIG.arena();
    private static final BallState DROPPED_BALL = new BallState(new Vec3(0, 0, 500), Vec3.ZERO);
    private static final CarState CAR_FACING_BALL =
            new CarState(new Vec3(-1000, 0, CONFIG.car().radius()), Vec3.ZERO, 0);

    @Test
    void ballNeverSinksBelowTheFloor() {
        World world = new World(CONFIG, DROPPED_BALL, List.of());

        for (int i = 0; i < 10 * CONFIG.ticksPerSecond(); i++) {
            world.step(List.of());
            assertThat(world.state().ball().position().z())
                    .isGreaterThanOrEqualTo(CONFIG.ball().radius());
        }
    }

    @Test
    void droppedBallEventuallyComesToRest() {
        World world = new World(CONFIG, DROPPED_BALL, List.of());

        for (int i = 0; i < 10 * CONFIG.ticksPerSecond(); i++) {
            world.step(List.of());
        }

        assertThat(world.state().ball().velocity()).isEqualTo(Vec3.ZERO);
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

        assertThat(first.state()).isEqualTo(second.state());
        assertThat(first.state().tick()).isEqualTo(1_000);
    }

    @Test
    void carDrivingIntoBallPushesItAndReportsTheTouch() {
        BallState restingBall = new BallState(new Vec3(0, 0, CONFIG.ball().radius()), Vec3.ZERO);
        World world = new World(CONFIG, restingBall, List.of(CAR_FACING_BALL));

        boolean touched = false;
        for (int i = 0; i < 2 * CONFIG.ticksPerSecond(); i++) {
            touched |= world.step(List.of(new CarInput(1, 0, false))).contains(new GameEvent.BallTouched(0));
        }

        assertThat(touched).isTrue();
        assertThat(world.state().ball().position().x()).isPositive();
    }

    @Test
    void ballCrossingTheLineScoresAndResetsForKickoff() {
        BallState aboutToScore = new BallState(new Vec3(ARENA.halfLength(), 0, 200), new Vec3(3000, 0, 0));
        World world = new World(CONFIG, aboutToScore, List.of(CAR_FACING_BALL));

        List<GameEvent> events = List.of();
        for (int i = 0; i < CONFIG.ticksPerSecond() && events.isEmpty(); i++) {
            events = world.step(List.of(CarInput.NEUTRAL));
        }

        assertThat(events).containsExactly(new GameEvent.GoalScored(Team.BLUE));
        assertThat(world.state().score()).isEqualTo(new Score(1, 0));
        assertThat(world.state().cars()).containsExactly(CAR_FACING_BALL);
    }

    @Test
    void ballBouncesOffTheEndWallBesideTheGoal() {
        double besideGoal = ARENA.goalHalfWidth() + 500;
        BallState ball = new BallState(new Vec3(ARENA.halfLength() - 200, besideGoal, 200), new Vec3(3000, 0, 0));
        World world = new World(CONFIG, ball, List.of());

        for (int i = 0; i < CONFIG.ticksPerSecond(); i++) {
            assertThat(world.step(List.of())).isEmpty();
        }

        assertThat(world.state().ball().velocity().x()).isNegative();
    }

    @Test
    void resetClearsScoreAndTick() {
        BallState aboutToScore = new BallState(new Vec3(ARENA.halfLength(), 0, 200), new Vec3(3000, 0, 0));
        World world = new World(CONFIG, aboutToScore, List.of());
        for (int i = 0; i < 30; i++) {
            world.step(List.of());
        }

        world.reset();

        assertThat(world.state().score()).isEqualTo(Score.ZERO);
        assertThat(world.state().tick()).isZero();
    }

    @Test
    void headOnKickoffIsFairAndKeepsTheBallUnderTheCeiling() {
        BallState centre = new BallState(new Vec3(0, 0, CONFIG.ball().radius()), Vec3.ZERO);
        double z = CONFIG.car().radius();
        World world = new World(
                CONFIG,
                centre,
                List.of(
                        new CarState(new Vec3(-2500, 0, z), Vec3.ZERO, 0),
                        new CarState(new Vec3(2500, 0, z), Vec3.ZERO, Math.PI)));
        List<CarInput> bothFullThrottle = List.of(new CarInput(1, 0, false), new CarInput(1, 0, false));

        for (int i = 0; i < 5 * CONFIG.ticksPerSecond(); i++) {
            world.step(bothFullThrottle);
            assertThat(world.state().ball().position().z())
                    .isLessThanOrEqualTo(ARENA.ceilingHeight() - CONFIG.ball().radius());
        }

        // Neither car gets an edge from being first in the list: the squeezed ball goes straight up.
        assertThat(world.state().ball().position().x()).isCloseTo(0, within(1e-6));
    }

    @Test
    void rejectsWrongNumberOfInputs() {
        World world = new World(CONFIG, DROPPED_BALL, List.of(CAR_FACING_BALL));

        assertThatIllegalArgumentException().isThrownBy(() -> world.step(List.of()));
    }
}
