package io.github.paultrde.carball.sim;

/**
 * Arcade car model. On the ground the tyres have perfect grip: the car only moves along its
 * heading, so any sideways velocity is dropped. In the air the car keeps its momentum and only
 * gravity applies.
 */
final class CarPhysics {

    private static final double GROUND_EPSILON = 1e-6;

    private final CarSpec spec;
    private final double gravity;

    CarPhysics(CarSpec spec, double gravity) {
        this.spec = spec;
        this.gravity = gravity;
    }

    CarState step(CarState car, CarInput input, double dt) {
        CarState driven = isGrounded(car) ? drive(car, input, dt) : car;
        return applyGravityAndFloor(driven, dt);
    }

    boolean isGrounded(CarState car) {
        return car.position().z() <= spec.radius() + GROUND_EPSILON;
    }

    private CarState drive(CarState car, CarInput input, double dt) {
        double forwardSpeed = car.velocity().dot(car.forward());
        double newSpeed =
                clamp(nextForwardSpeed(forwardSpeed, input.throttle(), dt), -spec.maxSpeed(), spec.maxSpeed());

        // Turning scales with speed, like a real car: no rotation at rest, mirrored in reverse.
        double yaw = car.yaw() + input.steer() * spec.turnRate() * (newSpeed / spec.maxSpeed()) * dt;
        Vec3 heading = new Vec3(Math.cos(yaw), Math.sin(yaw), 0);

        double verticalSpeed = input.jump() ? spec.jumpSpeed() : car.velocity().z();
        Vec3 velocity = heading.times(newSpeed).withZ(verticalSpeed);
        return new CarState(car.position(), velocity, yaw);
    }

    private double nextForwardSpeed(double speed, double throttle, double dt) {
        if (throttle != 0) {
            return speed + throttle * spec.acceleration() * dt;
        }
        // Coasting slows the car towards zero without overshooting into reverse.
        double slowdown = spec.coastDeceleration() * dt;
        return Math.signum(speed) * Math.max(0, Math.abs(speed) - slowdown);
    }

    private CarState applyGravityAndFloor(CarState car, double dt) {
        Vec3 velocity = car.velocity().plus(new Vec3(0, 0, gravity * dt));
        Vec3 position = car.position().plus(velocity.times(dt));

        if (position.z() < spec.radius()) {
            position = position.withZ(spec.radius());
            velocity = velocity.withZ(0);
        }
        return new CarState(position, velocity, car.yaw());
    }

    private static double clamp(double value, double min, double max) {
        return Math.max(min, Math.min(max, value));
    }
}
