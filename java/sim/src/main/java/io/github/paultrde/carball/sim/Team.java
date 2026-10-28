package io.github.paultrde.carball.sim;

/** BLUE defends the goal at -X and attacks +X; ORANGE does the opposite. */
public enum Team {
    BLUE(1),
    ORANGE(-1);

    private final int attackSign;

    Team(int attackSign) {
        this.attackSign = attackSign;
    }

    /** +1 if this team scores in the goal at +X, -1 if it scores at -X. */
    public int attackSign() {
        return attackSign;
    }

    /** The team that scores when the ball ends up on this side of the field. */
    public static Team scoringOnSide(double x) {
        return x > 0 ? BLUE : ORANGE;
    }
}
