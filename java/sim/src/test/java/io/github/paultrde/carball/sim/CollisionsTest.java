package io.github.paultrde.carball.sim;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.within;

import org.junit.jupiter.api.Test;

class CollisionsTest {

    private static final CarSpec CAR = CarSpec.DEFAULT;
    private static final BallSpec BALL = BallSpec.DEFAULT;

    @Test
    void ignoresSpheresThatDoNotTouch() {
        CarState car = new CarState(new Vec3(0, 0, 0), new Vec3(1000, 0, 0), 0);
        BallState ball = new BallState(new Vec3(500, 0, 0), Vec3.ZERO);

        Collisions.CarBallResult result = Collisions.carBall(car, CAR, ball, BALL);

        assertThat(result.car()).isEqualTo(car);
        assertThat(result.ball()).isEqualTo(ball);
    }

    @Test
    void carHittingRestingBallSendsItForwardAndConservesMomentum() {
        CarState car = new CarState(new Vec3(0, 0, 0), new Vec3(1000, 0, 0), 0);
        BallState ball = new BallState(new Vec3(140, 0, 0), Vec3.ZERO);

        Collisions.CarBallResult result = Collisions.carBall(car, CAR, ball, BALL);

        assertThat(result.ball().velocity().x()).isGreaterThan(1000);
        Vec3 before = momentum(car, ball);
        Vec3 after = momentum(result.car(), result.ball());
        assertThat(after.minus(before).length()).isCloseTo(0, within(1e-6));
    }

    @Test
    void separatesOverlappingSpheres() {
        CarState car = new CarState(new Vec3(0, 0, 0), Vec3.ZERO, 0);
        BallState ball = new BallState(new Vec3(100, 0, 0), Vec3.ZERO);

        Collisions.CarBallResult result = Collisions.carBall(car, CAR, ball, BALL);

        double distance =
                result.ball().position().minus(result.car().position()).length();
        assertThat(distance).isCloseTo(CAR.radius() + BALL.radius(), within(1e-9));
    }

    private static Vec3 momentum(CarState car, BallState ball) {
        return car.velocity().times(CAR.mass()).plus(ball.velocity().times(BALL.mass()));
    }
}
