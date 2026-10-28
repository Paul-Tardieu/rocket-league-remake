package io.github.paultrde.carball.sim;

/**
 * Keeps a sphere inside the arena, including inside a goal net once it has gone through the
 * opening. Callers decide how to react to each wall that was hit (bounce, stop...).
 */
final class ArenaBounds {

    private static final double EPSILON = 1e-9;

    record Confined(Vec3 position, boolean hitEndWall, boolean hitSideWall, boolean hitCeiling) {}

    private final ArenaSpec arena;

    ArenaBounds(ArenaSpec arena) {
        this.arena = arena;
    }

    /**
     * @param previous position at the start of the tick, used to know whether the sphere was already
     *     inside a goal net
     * @param next position after moving, possibly through a wall
     */
    Confined confine(Vec3 previous, Vec3 next, double radius) {
        double x = next.x();
        double y = next.y();
        double z = next.z();
        boolean hitEnd = false;
        boolean hitSide = false;
        boolean hitCeiling = false;

        double goalLine = arena.halfLength() - radius;
        if (Math.abs(x) > goalLine) {
            if (isInsideGoal(previous, radius) || fitsThroughGoalMouth(next, radius)) {
                double backWall = arena.halfLength() + arena.goalDepth() - radius;
                if (Math.abs(x) > backWall) {
                    x = Math.copySign(backWall, x);
                    hitEnd = true;
                }
                double netSide = arena.goalHalfWidth() - radius;
                if (Math.abs(y) > netSide) {
                    y = Math.copySign(netSide, y);
                    hitSide = true;
                }
                double roof = arena.goalHeight() - radius;
                if (z > roof) {
                    z = roof;
                    hitCeiling = true;
                }
            } else {
                x = Math.copySign(goalLine, x);
                hitEnd = true;
            }
        }

        double sideWall = arena.halfWidth() - radius;
        if (Math.abs(y) > sideWall) {
            y = Math.copySign(sideWall, y);
            hitSide = true;
        }
        return new Confined(new Vec3(x, y, z), hitEnd, hitSide, hitCeiling);
    }

    private boolean isInsideGoal(Vec3 position, double radius) {
        return Math.abs(position.x()) > arena.halfLength() - radius && fitsThroughGoalMouth(position, radius);
    }

    private boolean fitsThroughGoalMouth(Vec3 position, double radius) {
        return Math.abs(position.y()) <= arena.goalHalfWidth() - radius + EPSILON
                && position.z() <= arena.goalHeight() - radius + EPSILON;
    }
}
