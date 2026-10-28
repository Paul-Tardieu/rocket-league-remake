package io.github.paultrde.carball.sim;

/** Sphere-vs-sphere contact between a car hitbox and the ball, resolved with an impulse. */
final class Collisions {

    record CarBallResult(CarState car, BallState ball, boolean touched) {}

    private Collisions() {}

    static CarBallResult carBall(CarState car, CarSpec carSpec, BallState ball, BallSpec ballSpec) {
        Vec3 delta = ball.position().minus(car.position());
        double distance = delta.length();
        double penetration = carSpec.radius() + ballSpec.radius() - distance;
        if (penetration <= 0 || distance == 0) {
            return new CarBallResult(car, ball, false);
        }

        Vec3 normal = delta.times(1 / distance); // from car towards ball
        double inverseCarMass = 1 / carSpec.mass();
        double inverseBallMass = 1 / ballSpec.mass();
        double inverseMassSum = inverseCarMass + inverseBallMass;

        // 1. Push the spheres apart so they no longer overlap; the lighter one moves more.
        Vec3 correction = normal.times(penetration / inverseMassSum);
        Vec3 carPosition = car.position().minus(correction.times(inverseCarMass));
        Vec3 ballPosition = ball.position().plus(correction.times(inverseBallMass));

        // 2. If they are still moving towards each other, exchange an impulse along the normal.
        Vec3 carVelocity = car.velocity();
        Vec3 ballVelocity = ball.velocity();
        double approachSpeed = ballVelocity.minus(carVelocity).dot(normal);
        if (approachSpeed < 0) {
            double impulse = -(1 + ballSpec.restitution()) * approachSpeed / inverseMassSum;
            carVelocity = carVelocity.minus(normal.times(impulse * inverseCarMass));
            ballVelocity = ballVelocity.plus(normal.times(impulse * inverseBallMass));
        }

        return new CarBallResult(
                new CarState(carPosition, carVelocity, car.yaw()), new BallState(ballPosition, ballVelocity), true);
    }
}
