package io.github.paultrde.carball.sim;

/**
 * Rectangular arena centred on the origin, with a floor at z = 0 and a flat ceiling. Each end wall has a
 * goal opening centred on Y = 0 that leads into a box-shaped net behind the goal line.
 *
 * @param halfLength distance from the centre to the goal lines, along X
 * @param halfWidth distance from the centre to the side walls, along Y
 * @param goalHalfWidth half the width of a goal opening, along Y
 * @param goalHeight height of a goal opening
 * @param goalDepth depth of the net behind the goal line
 * @param ceilingHeight height of the arena ceiling
 */
public record ArenaSpec(
        double halfLength,
        double halfWidth,
        double goalHalfWidth,
        double goalHeight,
        double goalDepth,
        double ceilingHeight) {

    public static final ArenaSpec DEFAULT = new ArenaSpec(4000, 3000, 900, 640, 880, 2000);

    public ArenaSpec {
        Checks.positive("halfLength", halfLength);
        Checks.positive("halfWidth", halfWidth);
        Checks.inRange("goalHalfWidth", goalHalfWidth, Double.MIN_VALUE, halfWidth);
        Checks.positive("goalHeight", goalHeight);
        Checks.positive("goalDepth", goalDepth);
        Checks.inRange("ceilingHeight", ceilingHeight, goalHeight, Double.MAX_VALUE);
    }
}
