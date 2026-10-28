package io.github.paultrde.carball.render;

import io.github.paultrde.carball.sim.BallState;
import io.github.paultrde.carball.sim.CarState;
import io.github.paultrde.carball.sim.PhysicsConfig;
import io.github.paultrde.carball.sim.Vec3;
import io.github.paultrde.carball.sim.World;
import java.util.List;
import javax.swing.JFrame;
import javax.swing.SwingUtilities;
import javax.swing.Timer;

/**
 * Desktop entry point: one keyboard-driven car in a top-down Swing window.
 *
 * <p>Everything runs on the Swing event thread (the Timer fires there too), so the world is never
 * touched by two threads at once.
 */
public final class Main {

    private static final int FRAME_MILLIS = 16;

    private final PhysicsConfig config = PhysicsConfig.DEFAULT;
    private final KeyboardInput keyboard = new KeyboardInput();
    private final ArenaView view = new ArenaView(config);
    private World world = newMatch();
    private long lastFrameNanos = System.nanoTime();
    private double unsimulatedSeconds;

    private Main() {}

    public static void main(String[] args) {
        SwingUtilities.invokeLater(() -> new Main().start());
    }

    private void start() {
        JFrame frame = new JFrame("Carball");
        frame.setDefaultCloseOperation(JFrame.EXIT_ON_CLOSE);
        frame.add(view);
        frame.pack();
        frame.setLocationRelativeTo(null);
        frame.setVisible(true);
        view.addKeyListener(keyboard);
        view.requestFocusInWindow();

        new Timer(FRAME_MILLIS, event -> frame()).start();
    }

    /**
     * Fixed timestep with an accumulator: real elapsed time is banked, then spent in whole
     * simulation ticks. Rendering speed can vary, the physics always advances in identical steps.
     */
    private void frame() {
        long now = System.nanoTime();
        // Cap the catch-up so a long pause (debugger, window drag) does not trigger a burst of ticks.
        unsimulatedSeconds += Math.min((now - lastFrameNanos) / 1e9, 0.25);
        lastFrameNanos = now;

        if (keyboard.consumeReset()) {
            world = newMatch();
        }
        double dt = config.tickSeconds();
        while (unsimulatedSeconds >= dt) {
            world.step(List.of(keyboard.current()));
            unsimulatedSeconds -= dt;
        }
        view.show(world);
    }

    private World newMatch() {
        BallState ball = new BallState(new Vec3(0, 0, 400), Vec3.ZERO);
        CarState car = new CarState(new Vec3(-2000, 0, config.car().radius()), Vec3.ZERO, 0);
        return new World(config, ball, List.of(car));
    }
}
