package io.github.paultrde.carball.sim;

/**
 * Rectangular arena centred on the origin, with a floor at z = 0 and no ceiling.
 *
 * @param halfLength distance from the centre to the end walls, along X
 * @param halfWidth distance from the centre to the side walls, along Y
 */
public record ArenaSpec(double halfLength, double halfWidth) {

    public static final ArenaSpec DEFAULT = new ArenaSpec(4000, 3000);

    public ArenaSpec {
        Checks.positive("halfLength", halfLength);
        Checks.positive("halfWidth", halfWidth);
    }
}
