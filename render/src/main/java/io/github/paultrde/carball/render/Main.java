package io.github.paultrde.carball.render;

import io.github.paultrde.carball.sim.BallState;
import io.github.paultrde.carball.sim.PhysicsConfig;
import io.github.paultrde.carball.sim.Vec3;
import io.github.paultrde.carball.sim.World;

/** Placeholder entry point until a real renderer is chosen: prints the ball height each second. */
public final class Main {

    private Main() {}

    public static void main(String[] args) {
        PhysicsConfig config = PhysicsConfig.DEFAULT;
        World world = new World(config, new BallState(new Vec3(0, 0, 500), Vec3.ZERO));

        for (int second = 1; second <= 5; second++) {
            for (int i = 0; i < config.ticksPerSecond(); i++) {
                world.step();
            }
            System.out.printf(
                    "t=%ds ball z=%.1f%n", second, world.ball().position().z());
        }
    }
}
