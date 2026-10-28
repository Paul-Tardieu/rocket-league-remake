package io.github.paultrde.carball.sim;

import java.util.List;

/**
 * Immutable snapshot of a match, safe to hand to renderers and bots.
 *
 * @param cars cars in a stable order; a car's index is its identity
 */
public record GameState(BallState ball, List<CarState> cars, Score score, long tick) {

    public GameState {
        cars = List.copyOf(cars);
    }
}
