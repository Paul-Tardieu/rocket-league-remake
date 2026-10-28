package io.github.paultrde.carball.render;

import io.github.paultrde.carball.sim.ArenaSpec;
import io.github.paultrde.carball.sim.BallState;
import io.github.paultrde.carball.sim.CarState;
import io.github.paultrde.carball.sim.PhysicsConfig;
import io.github.paultrde.carball.sim.World;
import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Dimension;
import java.awt.Font;
import java.awt.Graphics;
import java.awt.Graphics2D;
import java.awt.RenderingHints;
import java.awt.geom.AffineTransform;
import java.awt.geom.Ellipse2D;
import java.awt.geom.Line2D;
import java.awt.geom.Point2D;
import java.awt.geom.Rectangle2D;
import java.awt.geom.RoundRectangle2D;
import java.io.Serial;
import java.util.List;
import javax.swing.JPanel;

/**
 * Top-down view of the arena. Height is faked: airborne objects are drawn shifted up the screen
 * and leave a shadow on the floor, which is enough to read jumps and ball arcs.
 */
final class ArenaView extends JPanel {

    @Serial
    private static final long serialVersionUID = 1L;

    private static final Color FLOOR = new Color(0x2E7D32);
    private static final Color LINES = new Color(255, 255, 255, 120);
    private static final Color SHADOW = new Color(0, 0, 0, 90);
    private static final Color BALL = new Color(0xECEFF1);
    private static final Color CAR = new Color(0x1E88E5);
    private static final Color CAR_NOSE = new Color(0xFFCA28);
    private static final double HEIGHT_SHIFT = 0.6;
    private static final int MARGIN = 30;
    private static final int HUD_HEIGHT = 50;

    private final transient PhysicsConfig config;
    private transient World world;

    ArenaView(PhysicsConfig config) {
        this.config = config;
        setPreferredSize(new Dimension(1200, 950));
        setBackground(new Color(0x1B1B1B));
        setFocusable(true);
    }

    void show(World snapshot) {
        this.world = snapshot;
        repaint();
    }

    @Override
    protected void paintComponent(Graphics graphics) {
        super.paintComponent(graphics);
        if (world == null) {
            return;
        }
        Graphics2D g = (Graphics2D) graphics.create();
        try {
            g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
            Projection projection = new Projection(config.arena(), getWidth(), getHeight());
            drawField(g, projection);
            world.cars()
                    .forEach(car -> drawShadow(
                            g,
                            projection,
                            car.position().x(),
                            car.position().y(),
                            config.car().radius()));
            drawShadow(
                    g,
                    projection,
                    world.ball().position().x(),
                    world.ball().position().y(),
                    config.ball().radius());
            world.cars().forEach(car -> drawCar(g, projection, car));
            drawBall(g, projection, world.ball());
            drawHud(g, world.cars(), world.ball());
        } finally {
            g.dispose();
        }
    }

    private void drawField(Graphics2D g, Projection p) {
        ArenaSpec arena = config.arena();
        Point2D topLeft = p.toScreen(-arena.halfLength(), arena.halfWidth(), 0);
        double width = 2 * arena.halfLength() * p.scale();
        double height = 2 * arena.halfWidth() * p.scale();

        g.setColor(FLOOR);
        g.fill(new Rectangle2D.Double(topLeft.getX(), topLeft.getY(), width, height));
        g.setColor(LINES);
        g.setStroke(new BasicStroke(3));
        g.draw(new Rectangle2D.Double(topLeft.getX(), topLeft.getY(), width, height));
        Point2D centre = p.toScreen(0, 0, 0);
        g.draw(new Line2D.Double(centre.getX(), topLeft.getY(), centre.getX(), topLeft.getY() + height));
        double circle = 1000 * p.scale();
        g.draw(new Ellipse2D.Double(centre.getX() - circle, centre.getY() - circle, 2 * circle, 2 * circle));
    }

    private void drawShadow(Graphics2D g, Projection p, double x, double y, double radius) {
        Point2D ground = p.toScreen(x, y, 0);
        double r = radius * p.scale();
        g.setColor(SHADOW);
        g.fill(new Ellipse2D.Double(ground.getX() - r, ground.getY() - r * 0.6, 2 * r, 1.2 * r));
    }

    private void drawBall(Graphics2D g, Projection p, BallState ball) {
        double floorOffset = ball.position().z() - config.ball().radius();
        Point2D centre = p.toScreen(ball.position().x(), ball.position().y(), floorOffset);
        double r = config.ball().radius() * p.scale();
        g.setColor(BALL);
        g.fill(new Ellipse2D.Double(centre.getX() - r, centre.getY() - r, 2 * r, 2 * r));
    }

    private void drawCar(Graphics2D g, Projection p, CarState car) {
        double floorOffset = car.position().z() - config.car().radius();
        Point2D centre = p.toScreen(car.position().x(), car.position().y(), floorOffset);
        double length = 2.4 * config.car().radius() * p.scale();
        double width = 1.4 * config.car().radius() * p.scale();

        AffineTransform saved = g.getTransform();
        g.translate(centre.getX(), centre.getY());
        g.rotate(-car.yaw()); // screen Y points down, so world counter-clockwise is screen clockwise
        g.setColor(CAR);
        g.fill(new RoundRectangle2D.Double(-length / 2, -width / 2, length, width, width / 2, width / 2));
        g.setColor(CAR_NOSE);
        g.fill(new Rectangle2D.Double(length / 2 - length / 5, -width / 2, length / 5, width));
        g.setTransform(saved);
    }

    private void drawHud(Graphics2D g, List<CarState> cars, BallState ball) {
        g.setColor(Color.WHITE);
        g.setFont(new Font(Font.MONOSPACED, Font.PLAIN, 14));
        int baseline = getHeight() - HUD_HEIGHT / 2;
        String carText = cars.isEmpty()
                ? ""
                : "car speed %6.0f  ".formatted(cars.getFirst().velocity().length());
        g.drawString(
                carText
                        + "ball height %5.0f  speed %6.0f  |  tick %d"
                                .formatted(ball.position().z(), ball.velocity().length(), world.tick()),
                MARGIN,
                baseline);
        g.drawString("W/S throttle   A/D steer   SPACE jump   R reset", MARGIN, baseline + 18);
    }

    /** Maps world units (X right, Y up, Z height) to screen pixels, fitting the arena in the panel. */
    private record Projection(double scale, double centreX, double centreY) {

        Projection(ArenaSpec arena, int panelWidth, int panelHeight) {
            this(
                    Math.min(
                            (panelWidth - 2.0 * MARGIN) / (2 * arena.halfLength()),
                            (panelHeight - 2.0 * MARGIN - HUD_HEIGHT) / (2 * arena.halfWidth())),
                    panelWidth / 2.0,
                    (panelHeight - HUD_HEIGHT) / 2.0);
        }

        Point2D toScreen(double x, double y, double height) {
            return new Point2D.Double(centreX + x * scale, centreY - (y + height * HEIGHT_SHIFT) * scale);
        }
    }
}
