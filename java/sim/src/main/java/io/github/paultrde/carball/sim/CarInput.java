package io.github.paultrde.carball.sim;

/**
 * What a player or bot asks the car to do during one tick. This is the bot's action space.
 *
 * @param throttle -1 full reverse, 1 full forward
 * @param steer -1 full right, 1 full left (positive yaw is counter-clockwise seen from above)
 * @param jump whether to jump; ignored while airborne
 */
public record CarInput(double throttle, double steer, boolean jump) {

    public static final CarInput NEUTRAL = new CarInput(0, 0, false);

    public CarInput {
        Checks.inRange("throttle", throttle, -1, 1);
        Checks.inRange("steer", steer, -1, 1);
    }
}
