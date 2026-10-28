package io.github.paultrde.carball.bot;

import io.github.paultrde.carball.sim.ArenaSpec;
import io.github.paultrde.carball.sim.CarInput;
import io.github.paultrde.carball.sim.CarState;
import io.github.paultrde.carball.sim.GameState;
import io.github.paultrde.carball.sim.Team;
import io.github.paultrde.carball.sim.Vec3;

/**
 * Hand-written baseline: get behind the ball, then drive through it towards the opponent goal.
 *
 * <p>The trick is the aim point. Driving straight at the ball pushes it wherever the car happens
 * to come from. Instead the bot aims at a point behind the ball on the goal-to-ball line, and that
 * point slides onto the ball as the car gets closer. When the car is on the wrong side of the ball,
 * it first swings wide around it so it does not knock the ball towards its own goal on the way.
 */
public final class StrikerBot implements Bot {

    /** Aim offset behind the ball, as a fraction of the car-to-ball distance. */
    private static final double OFFSET_RATIO = 0.5;
    /** Cap on the aim offset, in units. */
    private static final double MAX_OFFSET = 1500;
    /** Distance behind and beside the ball used to swing around it from the wrong side. */
    private static final double DETOUR_DISTANCE = 900;
    /** How far in front of the ball (towards its target goal) the car may be before detouring. */
    private static final double GOAL_SIDE_TOLERANCE = 100;
    /** Converts a heading error in radians into a steer value; higher turns more aggressively. */
    private static final double STEER_GAIN = 5;
    /** Heading error above which the bot slows down to turn tighter, in radians. */
    private static final double SHARP_TURN = Math.toRadians(45);
    /** Heading error above which, at low speed, backing up turns the car round faster. */
    private static final double TARGET_BEHIND = Math.toRadians(110);
    /** Backwards speed above which the car counts as reversing, in units/s. */
    private static final double REVERSE_SPEED_EPSILON = 50;
    /** Speed the bot brakes down to for sharp turns; low speed means a small turning radius. */
    private static final double CORNERING_SPEED = 1000;
    /** Keeps aim points this far from the walls so the car does not grind along them. */
    private static final double WALL_MARGIN = 250;

    private final int carIndex;
    private final ArenaSpec arena;
    private final Vec3 targetGoal;

    public StrikerBot(int carIndex, Team team, ArenaSpec arena) {
        this.carIndex = carIndex;
        this.arena = arena;
        this.targetGoal = new Vec3(team.attackSign() * arena.halfLength(), 0, 0);
    }

    @Override
    public CarInput act(GameState state) {
        CarState car = state.cars().get(carIndex);
        Vec3 aim = chooseAim(flat(car.position()), flat(state.ball().position()));
        return driveTowards(car, aim);
    }

    private Vec3 chooseAim(Vec3 car, Vec3 ball) {
        Vec3 ballToGoal = unit(targetGoal.minus(ball));
        Vec3 ballToCar = car.minus(ball);

        // Aim behind the ball, further behind the further away the car is. The aim slides onto the
        // ball as the car closes in, which bends the path into a curve that arrives lined up with
        // the goal instead of hitting the ball side-on.
        double offset = Math.min(ballToCar.length() * OFFSET_RATIO, MAX_OFFSET);
        Vec3 aim = ball.minus(ballToGoal.times(offset));

        if (ballToCar.dot(ballToGoal) > -GOAL_SIDE_TOLERANCE) {
            // Car is on the goal side: swing wide around the ball on whichever side the car is.
            Vec3 side = new Vec3(-ballToGoal.y(), ballToGoal.x(), 0);
            double sideSign = ballToCar.dot(side) >= 0 ? 1 : -1;
            aim = ball.minus(ballToGoal.times(DETOUR_DISTANCE)).plus(side.times(sideSign * DETOUR_DISTANCE));
        }
        return keepOffWalls(aim);
    }

    private static CarInput driveTowards(CarState car, Vec3 aim) {
        double error = headingError(car, aim);
        double forwardSpeed = car.velocity().dot(car.forward());

        // Steering is mirrored whenever the car actually rolls backwards, whatever the throttle.
        double steer = forwardSpeed < 0 ? -steerFor(error) : steerFor(error);

        // Backing up with the aim behind is a three-point turn; it also frees a car stuck nose-first
        // against a wall. Hysteresis: start reversing only when the aim is well behind, but once
        // rolling backwards keep going until the nose roughly points at the aim. With a single
        // threshold the bot would flip between forward and reverse every tick.
        boolean rollingBackwards = forwardSpeed < -REVERSE_SPEED_EPSILON;
        boolean reverse = rollingBackwards
                ? Math.abs(error) > SHARP_TURN
                : Math.abs(error) > TARGET_BEHIND && forwardSpeed < CORNERING_SPEED;
        if (reverse) {
            return new CarInput(-1, steer, false);
        }
        boolean tooFastToTurn = Math.abs(error) > SHARP_TURN && forwardSpeed > CORNERING_SPEED;
        return new CarInput(tooFastToTurn ? -1 : 1, steer, false);
    }

    private static double steerFor(double headingError) {
        return Math.clamp(headingError * STEER_GAIN, -1, 1);
    }

    /** Signed angle from the car's heading to the aim point, in [-π, π]. */
    private static double headingError(CarState car, Vec3 aim) {
        double wanted = Math.atan2(
                aim.y() - car.position().y(), aim.x() - car.position().x());
        return wrapAngle(wanted - car.yaw());
    }

    private Vec3 keepOffWalls(Vec3 point) {
        double maxX = arena.halfLength() - WALL_MARGIN;
        double maxY = arena.halfWidth() - WALL_MARGIN;
        return new Vec3(Math.clamp(point.x(), -maxX, maxX), Math.clamp(point.y(), -maxY, maxY), 0);
    }

    /** Brings an angle back into [-π, π] so the bot always turns the short way round. */
    private static double wrapAngle(double radians) {
        return Math.atan2(Math.sin(radians), Math.cos(radians));
    }

    private static Vec3 flat(Vec3 v) {
        return v.withZ(0);
    }

    private static Vec3 unit(Vec3 v) {
        double length = v.length();
        return length == 0 ? Vec3.ZERO : v.times(1 / length);
    }
}
