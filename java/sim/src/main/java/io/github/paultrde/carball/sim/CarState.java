package io.github.paultrde.carball.sim;

/**
 * @param yaw heading in radians, 0 faces +X, counter-clockwise seen from above
 */
public record CarState(Vec3 position, Vec3 velocity, double yaw) {

    public Vec3 forward() {
        return new Vec3(Math.cos(yaw), Math.sin(yaw), 0);
    }
}
