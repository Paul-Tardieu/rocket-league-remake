package io.github.paultrde.carball.sim;

import java.util.ArrayList;
import java.util.List;

/**
 * Deterministic fixed-timestep simulation. The same initial state and the same sequence of
 * inputs always produce the same result, which is required for bot training.
 */
public final class World {

    private final PhysicsConfig config;
    private final BallPhysics ballPhysics;
    private final CarPhysics carPhysics;
    private BallState ball;
    private List<CarState> cars;
    private long tick;

    public World(PhysicsConfig config, BallState initialBall, List<CarState> initialCars) {
        this.config = config;
        this.ballPhysics = new BallPhysics(config);
        this.carPhysics = new CarPhysics(config);
        this.ball = initialBall;
        this.cars = List.copyOf(initialCars);
    }

    /**
     * Advances the simulation by one tick.
     *
     * @param inputs one input per car, in the same order as {@link #cars()}
     */
    public void step(List<CarInput> inputs) {
        if (inputs.size() != cars.size()) {
            throw new IllegalArgumentException("expected " + cars.size() + " inputs, got " + inputs.size());
        }
        double dt = config.tickSeconds();

        BallState nextBall = ballPhysics.step(ball, dt);
        List<CarState> nextCars = new ArrayList<>(cars.size());
        for (int i = 0; i < cars.size(); i++) {
            CarState car = carPhysics.step(cars.get(i), inputs.get(i), dt);
            Collisions.CarBallResult contact = Collisions.carBall(car, config.car(), nextBall, config.ball());
            nextBall = contact.ball();
            nextCars.add(contact.car());
        }

        ball = nextBall;
        cars = List.copyOf(nextCars);
        tick++;
    }

    public BallState ball() {
        return ball;
    }

    /** Unmodifiable snapshot of the cars, in a stable order. */
    public List<CarState> cars() {
        return cars;
    }

    public long tick() {
        return tick;
    }
}
