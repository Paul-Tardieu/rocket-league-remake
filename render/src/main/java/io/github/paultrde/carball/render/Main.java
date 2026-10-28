package io.github.paultrde.carball.render;

import io.github.paultrde.carball.sim.BallState;
import io.github.paultrde.carball.sim.CarInput;
import io.github.paultrde.carball.sim.CarState;
import io.github.paultrde.carball.sim.PhysicsConfig;
import io.github.paultrde.carball.sim.Vec3;
import io.github.paultrde.carball.sim.World;
import java.util.List;

/** Placeholder entry point until a real renderer is chosen: a car drives into the ball. */
public final class Main {

    private Main() {}

    public static void main(String[] args) {
        PhysicsConfig config = PhysicsConfig.DEFAULT;
        BallState ball = new BallState(new Vec3(0, 0, config.ball().radius()), Vec3.ZERO);
        CarState car = new CarState(new Vec3(-1500, 0, config.car().radius()), Vec3.ZERO, 0);
        World world = new World(config, ball, List.of(car));
        List<CarInput> fullThrottle = List.of(new CarInput(1, 0, false));

        int ticksPerPrint = config.ticksPerSecond() / 4;
        for (int print = 1; print <= 12; print++) {
            for (int i = 0; i < ticksPerPrint; i++) {
                world.step(fullThrottle);
            }
            CarState c = world.cars().getFirst();
            BallState b = world.ball();
            System.out.printf(
                    "t=%.2fs  car x=%7.1f v=%6.1f | ball x=%7.1f z=%5.1f v=%6.1f%n",
                    world.tick() * config.tickSeconds(),
                    c.position().x(),
                    c.velocity().length(),
                    b.position().x(),
                    b.position().z(),
                    b.velocity().length());
        }
    }
}
