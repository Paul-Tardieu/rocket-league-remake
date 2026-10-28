package io.github.paultrde.carball.render;

import io.github.paultrde.carball.bot.Bot;
import io.github.paultrde.carball.bot.StrikerBot;
import io.github.paultrde.carball.sim.BallState;
import io.github.paultrde.carball.sim.CarInput;
import io.github.paultrde.carball.sim.CarState;
import io.github.paultrde.carball.sim.GameEvent;
import io.github.paultrde.carball.sim.PhysicsConfig;
import io.github.paultrde.carball.sim.Team;
import io.github.paultrde.carball.sim.Vec3;
import io.github.paultrde.carball.sim.World;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import javax.swing.JFrame;
import javax.swing.SwingUtilities;
import javax.swing.Timer;

/**
 * Desktop entry point: you (BLUE, keyboard) against a {@link StrikerBot} (ORANGE). Run with
 * {@code --watch} to watch two bots play each other instead.
 *
 * <p>Everything runs on the Swing event thread (the Timer fires there too), so the world is never
 * touched by two threads at once.
 */
public final class Main {

    private static final int FRAME_MILLIS = 16;
    private static final double GOAL_BANNER_SECONDS = 1.5;
    private static final List<Team> TEAMS = List.of(Team.BLUE, Team.ORANGE);

    private final PhysicsConfig config = PhysicsConfig.DEFAULT;
    private final KeyboardInput keyboard = new KeyboardInput();
    private final List<Bot> controllers;
    private final ArenaView view;
    private final World world;
    private long lastFrameNanos = System.nanoTime();
    private double unsimulatedSeconds;
    private Optional<Team> lastScorer = Optional.empty();
    private double bannerSecondsLeft;

    private Main(boolean watchBots) {
        // The keyboard is just another Bot: something that turns a GameState into a CarInput.
        Bot blue = watchBots ? new StrikerBot(0, Team.BLUE, config.arena()) : state -> keyboard.current();
        Bot orange = new StrikerBot(1, Team.ORANGE, config.arena());
        this.controllers = List.of(blue, orange);
        String help =
                watchBots ? "Bot vs bot   R reset" : "You are BLUE   W/S throttle   A/D steer   SPACE jump   R reset";
        this.view = new ArenaView(config, TEAMS, help);
        this.world = new World(config, kickoffBall(), kickoffCars());
    }

    public static void main(String[] args) {
        boolean watchBots = List.of(args).contains("--watch");
        SwingUtilities.invokeLater(() -> new Main(watchBots).start());
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
        double elapsed = Math.min((now - lastFrameNanos) / 1e9, 0.25);
        unsimulatedSeconds += elapsed;
        lastFrameNanos = now;
        bannerSecondsLeft = Math.max(0, bannerSecondsLeft - elapsed);

        if (keyboard.consumeReset()) {
            world.reset();
            bannerSecondsLeft = 0;
        }
        double dt = config.tickSeconds();
        while (unsimulatedSeconds >= dt) {
            tick();
            unsimulatedSeconds -= dt;
        }
        view.show(world.state(), bannerSecondsLeft > 0 ? lastScorer : Optional.empty());
    }

    private void tick() {
        List<CarInput> inputs = new ArrayList<>(controllers.size());
        for (Bot controller : controllers) {
            inputs.add(controller.act(world.state()));
        }
        for (GameEvent event : world.step(inputs)) {
            switch (event) {
                case GameEvent.GoalScored(Team scorer) -> {
                    lastScorer = Optional.of(scorer);
                    bannerSecondsLeft = GOAL_BANNER_SECONDS;
                }
                case GameEvent.BallTouched touched -> {
                    // Nothing to show yet; bots will use touches as a training signal.
                }
            }
        }
    }

    private BallState kickoffBall() {
        return new BallState(new Vec3(0, 0, config.ball().radius()), Vec3.ZERO);
    }

    private List<CarState> kickoffCars() {
        double z = config.car().radius();
        return List.of(
                new CarState(new Vec3(-2500, 0, z), Vec3.ZERO, 0),
                new CarState(new Vec3(2500, 0, z), Vec3.ZERO, Math.PI));
    }
}
