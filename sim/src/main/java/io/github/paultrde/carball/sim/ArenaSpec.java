package io.github.paultrde.carball.sim;

/**
 * Rectangular arena centred on the origin, with a floor at z = 0 and no ceiling. Each end wall has a
 * goal opening centred on Y = 0 that leads into a box-shaped net behind the goal line.
 *
 * @param halfLength distance from the centre to the goal lines, along X
 * @param halfWidth distance from the centre to the side walls, along Y
 * @param goalHalfWidth half the width of a goal opening, along Y
 * @param goalHeight height of a goal opening
 * @param goalDepth depth of the net behind the goal line
 */
public record ArenaSpec(
        double halfLength, double halfWidth, double goalHalfWidth, double goalHeight, double goalDepth) {

    public static final ArenaSpec DEFAULT = new ArenaSpec(4000, 3000, 900, 640, 880);

    public ArenaSpec {
        Checks.positive("halfLength", halfLength);
        Checks.positive("halfWidth", halfWidth);
        Checks.inRange("goalHalfWidth", goalHalfWidth, Double.MIN_VALUE, halfWidth);
        Checks.positive("goalHeight", goalHeight);
        Checks.positive("goalDepth", goalDepth);
    }
}
