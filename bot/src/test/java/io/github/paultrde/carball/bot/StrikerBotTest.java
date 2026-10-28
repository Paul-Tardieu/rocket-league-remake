package io.github.paultrde.carball.bot;

import static org.assertj.core.api.Assertions.assertThat;

import io.github.paultrde.carball.sim.ArenaSpec;
import io.github.paultrde.carball.sim.BallState;
import io.github.paultrde.carball.sim.CarState;
import io.github.paultrde.carball.sim.GameEvent;
import io.github.paultrde.carball.sim.PhysicsConfig;
import io.github.paultrde.carball.sim.Team;
import io.github.paultrde.carball.sim.Vec3;
import io.github.paultrde.carball.sim.World;
import java.util.List;
import java.util.Optional;
import java.util.Random;
import org.junit.jupiter.api.Test;

class StrikerBotTest {

    private static final PhysicsConfig CONFIG = PhysicsConfig.DEFAULT;
    private static final ArenaSpec ARENA = CONFIG.arena();
    private static final int MAX_SECONDS = 30;

    @Test
    void scoresFromKickoff() {
        World world = new World(CONFIG, ballAt(0, 0), List.of(carAt(-2000, 0, 0)));

        assertThat(firstGoal(world, new StrikerBot(0, Team.BLUE, ARENA))).contains(Team.BLUE);
    }

    @Test
    void orangeBotAttacksTheOtherWay() {
        World world = new World(CONFIG, ballAt(0, 0), List.of(carAt(2000, 0, Math.PI)));

        assertThat(firstGoal(world, new StrikerBot(0, Team.ORANGE, ARENA))).contains(Team.ORANGE);
    }

    /**
     * The bot is a heuristic, so single hand-picked cases are a poor test: it fails on some hard
     * spots (ball stuck in a corner) by design. Instead, measure its success rate over many random
     * but reproducible starts (fixed seed), and fail if it regresses.
     */
    @Test
    void scoresFromMostRandomStartsWithoutOwnGoals() {
        Random random = new Random(1234);
        int goals = 0;
        int ownGoals = 0;
        int runs = 100;
        for (int run = 0; run < runs; run++) {
            World world = new World(
                    CONFIG,
                    ballAt(randomX(random), randomY(random)),
                    List.of(carAt(randomX(random), randomY(random), random.nextDouble() * 2 * Math.PI)));
            Optional<Team> scorer = firstGoal(world, new StrikerBot(0, Team.BLUE, ARENA));
            if (scorer.equals(Optional.of(Team.BLUE))) {
                goals++;
            } else if (scorer.isPresent()) {
                ownGoals++;
            }
        }

        assertThat(goals).as("goals out of %d starts", runs).isGreaterThanOrEqualTo(80);
        assertThat(ownGoals).as("own goals out of %d starts", runs).isLessThanOrEqualTo(2);
    }

    private static Optional<Team> firstGoal(World world, Bot bot) {
        for (int tick = 0; tick < MAX_SECONDS * CONFIG.ticksPerSecond(); tick++) {
            for (GameEvent event : world.step(List.of(bot.act(world.state())))) {
                if (event instanceof GameEvent.GoalScored(Team scorer)) {
                    return Optional.of(scorer);
                }
            }
        }
        return Optional.empty();
    }

    private static BallState ballAt(double x, double y) {
        return new BallState(new Vec3(x, y, CONFIG.ball().radius()), Vec3.ZERO);
    }

    private static CarState carAt(double x, double y, double yaw) {
        return new CarState(new Vec3(x, y, CONFIG.car().radius()), Vec3.ZERO, yaw);
    }

    private static double randomX(Random random) {
        return (random.nextDouble() * 2 - 1) * (ARENA.halfLength() - 400);
    }

    private static double randomY(Random random) {
        return (random.nextDouble() * 2 - 1) * (ARENA.halfWidth() - 300);
    }
}
