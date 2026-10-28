package io.github.paultrde.carball.sim;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.within;

import org.junit.jupiter.api.Test;

class CarPhysicsTest {

    private static final CarSpec SPEC = CarSpec.DEFAULT;
    private static final double DT = PhysicsConfig.DEFAULT.tickSeconds();
    private static final CarState PARKED = new CarState(new Vec3(0, 0, SPEC.radius()), Vec3.ZERO, 0);

    private final CarPhysics physics = new CarPhysics(SPEC, PhysicsConfig.DEFAULT.gravity());

    @Test
    void fullThrottleNeverExceedsMaxSpeed() {
        CarState car = run(PARKED, new CarInput(1, 0, false), 10);

        assertThat(car.velocity().length()).isCloseTo(SPEC.maxSpeed(), within(1e-9));
        assertThat(car.velocity().x()).isPositive();
    }

    @Test
    void cannotTurnWhileStopped() {
        CarState car = run(PARKED, new CarInput(0, 1, false), 1);

        assertThat(car.yaw()).isZero();
    }

    @Test
    void steersLeftWhenMovingForward() {
        CarState moving = run(PARKED, new CarInput(1, 0, false), 1);

        CarState turned = run(moving, new CarInput(1, 1, false), 0.5);

        assertThat(turned.yaw()).isPositive();
    }

    @Test
    void coastingStopsWithoutRollingBackwards() {
        CarState moving = run(PARKED, new CarInput(1, 0, false), 1);

        CarState stopped = run(moving, CarInput.NEUTRAL, 10);

        assertThat(stopped.velocity().length()).isZero();
    }

    @Test
    void jumpsOnlyFromTheGround() {
        CarState airborne = physics.step(PARKED, new CarInput(0, 0, true), DT);
        assertThat(physics.isGrounded(airborne)).isFalse();

        CarState stillJumping = physics.step(airborne, new CarInput(0, 0, true), DT);

        assertThat(stillJumping.velocity().z()).isLessThan(airborne.velocity().z());
    }

    private CarState run(CarState start, CarInput input, double seconds) {
        CarState car = start;
        for (int i = 0; i < Math.round(seconds / DT); i++) {
            car = physics.step(car, input, DT);
        }
        return car;
    }
}
