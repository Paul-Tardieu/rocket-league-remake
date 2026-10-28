package io.github.paultrde.carball.sim;

public record Score(int blue, int orange) {

    public static final Score ZERO = new Score(0, 0);

    public Score plusGoalFor(Team team) {
        return switch (team) {
            case BLUE -> new Score(blue + 1, orange);
            case ORANGE -> new Score(blue, orange + 1);
        };
    }
}
