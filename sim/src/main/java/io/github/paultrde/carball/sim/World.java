package io.github.paultrde.carball.sim;

import java.util.ArrayList;
import java.util.List;

/**
 * Deterministic fixed-timestep simulation of a match. The same kickoff and the same sequence of
 * inputs always produce the same result, which is required for bot training.
 */
public final class World {

    private final PhysicsConfig config;
    private final BallPhysics ballPhysics;
    private final CarPhysics carPhysics;
    private final BallState kickoffBall;
    private final List<CarState> kickoffCars;
    private BallState ball;
    private List<CarState> cars;
    private Score score = Score.ZERO;
    private long tick;

    /** Starts a match; the given positions are also used for every kickoff after a goal. */
    public World(PhysicsConfig config, BallState kickoffBall, List<CarState> kickoffCars) {
        this.config = config;
        this.ballPhysics = new BallPhysics(config);
        this.carPhysics = new CarPhysics(config);
        this.kickoffBall = kickoffBall;
        this.kickoffCars = List.copyOf(kickoffCars);
        placeForKickoff();
    }

    /**
     * Advances the simulation by one tick.
     *
     * @param inputs one input per car, in the same order as {@link GameState#cars()}
     * @return what happened during this tick, in order
     */
    public List<GameEvent> step(List<CarInput> inputs) {
        if (inputs.size() != cars.size()) {
            throw new IllegalArgumentException("expected " + cars.size() + " inputs, got " + inputs.size());
        }
        double dt = config.tickSeconds();
        List<GameEvent> events = new ArrayList<>();

        BallState nextBall = ballPhysics.step(ball, dt);
        List<CarState> nextCars = new ArrayList<>(cars.size());
        for (int i = 0; i < cars.size(); i++) {
            CarState car = carPhysics.step(cars.get(i), inputs.get(i), dt);
            Collisions.CarBallResult contact = Collisions.carBall(car, config.car(), nextBall, config.ball());
            if (contact.touched()) {
                events.add(new GameEvent.BallTouched(i));
            }
            nextBall = contact.ball();
            nextCars.add(contact.car());
        }
        ball = nextBall;
        cars = List.copyOf(nextCars);
        tick++;

        if (isGoal(ball)) {
            Team scorer = Team.scoringOnSide(ball.position().x());
            score = score.plusGoalFor(scorer);
            events.add(new GameEvent.GoalScored(scorer));
            placeForKickoff();
        }
        return List.copyOf(events);
    }

    /** Starts a fresh match: kickoff positions, score 0-0, tick 0. */
    public void reset() {
        placeForKickoff();
        score = Score.ZERO;
        tick = 0;
    }

    public GameState state() {
        return new GameState(ball, cars, score, tick);
    }

    /** A goal counts once the whole ball is past the goal line. */
    private boolean isGoal(BallState ball) {
        return Math.abs(ball.position().x())
                > config.arena().halfLength() + config.ball().radius();
    }

    private void placeForKickoff() {
        ball = kickoffBall;
        cars = kickoffCars;
    }
}
