package io.github.paultrde.carball.sim;

/** Argument validation shared by the record constructors of this package. */
final class Checks {

    private Checks() {}

    static void positive(String name, double value) {
        if (!(value > 0)) {
            throw new IllegalArgumentException(name + " must be positive: " + value);
        }
    }

    static void inRange(String name, double value, double min, double max) {
        if (!(value >= min && value <= max)) {
            throw new IllegalArgumentException(name + " must be in [" + min + ", " + max + "]: " + value);
        }
    }
}
