package io.github.paultrde.carball.sim;

/**
 * Something notable that happened during a tick. Sealed so a {@code switch} over events is checked
 * by the compiler: adding a new event type breaks every switch that forgot to handle it.
 */
public sealed interface GameEvent {

    /** The ball fully crossed a goal line; positions have already been reset for kickoff. */
    record GoalScored(Team scorer) implements GameEvent {}

    /** A car hit the ball. Useful as a reward signal for bots. */
    record BallTouched(int carIndex) implements GameEvent {}
}
