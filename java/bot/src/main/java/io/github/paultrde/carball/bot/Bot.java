package io.github.paultrde.carball.bot;

import io.github.paultrde.carball.sim.CarInput;
import io.github.paultrde.carball.sim.GameState;

/** Controls one car. Called once per simulation tick with the latest state. */
@FunctionalInterface
public interface Bot {

    CarInput act(GameState state);
}
