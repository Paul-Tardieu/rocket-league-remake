package io.github.paultrde.carball.render;

import io.github.paultrde.carball.sim.ArenaSpec;
import io.github.paultrde.carball.sim.BallState;
import io.github.paultrde.carball.sim.CarState;
import io.github.paultrde.carball.sim.GameState;
import io.github.paultrde.carball.sim.PhysicsConfig;
import io.github.paultrde.carball.sim.Team;
import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Dimension;
import java.awt.Font;
import java.awt.FontMetrics;
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
import java.util.Optional;
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
    private static final Color BLUE = new Color(0x1E88E5);
    private static final Color ORANGE = new Color(0xFB8C00);
    private static final Color CAR_NOSE = new Color(0xFFEB3B);
    private static final double HEIGHT_SHIFT = 0.6;
    private static final int MARGIN = 30;
    private static final int SCORE_HEIGHT = 50;
    private static final int HUD_HEIGHT = 50;

    private final transient PhysicsConfig config;
    private final transient List<Team> carTeams;
    private final String controlsHelp;
    private transient GameState state;
    private transient Optional<Team> goalBanner = Optional.empty();

    /**
     * @param carTeams team of each car, in the same order as {@link GameState#cars()}
     * @param controlsHelp one line shown at the bottom of the window
     */
    ArenaView(PhysicsConfig config, List<Team> carTeams, String controlsHelp) {
        this.config = config;
        this.carTeams = List.copyOf(carTeams);
        this.controlsHelp = controlsHelp;
        setPreferredSize(new Dimension(1300, 950));
        setBackground(new Color(0x1B1B1B));
        setFocusable(true);
    }

    /**
     * @param goalBanner team to celebrate on screen, if a goal was scored moments ago
     */
    void show(GameState snapshot, Optional<Team> goalBanner) {
        this.state = snapshot;
        this.goalBanner = goalBanner;
        repaint();
    }

    @Override
    protected void paintComponent(Graphics graphics) {
        super.paintComponent(graphics);
        if (state == null) {
            return;
        }
        Graphics2D g = (Graphics2D) graphics.create();
        try {
            g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
            Projection projection = new Projection(config.arena(), getWidth(), getHeight());
            drawField(g, projection);
            for (CarState car : state.cars()) {
                drawShadow(
                        g,
                        projection,
                        car.position().x(),
                        car.position().y(),
                        config.car().radius());
            }
            BallState ball = state.ball();
            drawShadow(
                    g,
                    projection,
                    ball.position().x(),
                    ball.position().y(),
                    config.ball().radius());
            for (int i = 0; i < state.cars().size(); i++) {
                drawCar(g, projection, state.cars().get(i), colorOf(carTeams.get(i)));
            }
            drawBall(g, projection, ball);
            drawScore(g);
            drawHud(g);
            goalBanner.ifPresent(team -> drawGoalBanner(g, team));
        } finally {
            g.dispose();
        }
    }

    private void drawField(Graphics2D g, Projection p) {
        ArenaSpec arena = config.arena();
        // BLUE defends the goal at -X, ORANGE the one at +X.
        drawGoal(g, p, -1, BLUE);
        drawGoal(g, p, 1, ORANGE);

        Rectangle2D field = p.rectangle(-arena.halfLength(), arena.halfWidth(), arena.halfLength(), -arena.halfWidth());
        g.setColor(FLOOR);
        g.fill(field);
        g.setColor(LINES);
        g.setStroke(new BasicStroke(3));
        g.draw(field);
        Point2D centre = p.toScreen(0, 0, 0);
        g.draw(new Line2D.Double(centre.getX(), field.getMinY(), centre.getX(), field.getMaxY()));
        double circle = 1000 * p.scale();
        g.draw(new Ellipse2D.Double(centre.getX() - circle, centre.getY() - circle, 2 * circle, 2 * circle));
    }

    /** @param side -1 for the goal at -X, +1 for the goal at +X */
    private void drawGoal(Graphics2D g, Projection p, int side, Color color) {
        ArenaSpec arena = config.arena();
        double goalLine = side * arena.halfLength();
        double backWall = side * (arena.halfLength() + arena.goalDepth());
        Rectangle2D net = p.rectangle(
                Math.min(goalLine, backWall),
                arena.goalHalfWidth(),
                Math.max(goalLine, backWall),
                -arena.goalHalfWidth());
        g.setColor(color.darker());
        g.fill(net);
        g.setColor(color);
        g.setStroke(new BasicStroke(3));
        g.draw(net);
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

    private void drawCar(Graphics2D g, Projection p, CarState car, Color color) {
        double floorOffset = car.position().z() - config.car().radius();
        Point2D centre = p.toScreen(car.position().x(), car.position().y(), floorOffset);
        double length = 2.4 * config.car().radius() * p.scale();
        double width = 1.4 * config.car().radius() * p.scale();

        AffineTransform saved = g.getTransform();
        g.translate(centre.getX(), centre.getY());
        g.rotate(-car.yaw()); // screen Y points down, so world counter-clockwise is screen clockwise
        g.setColor(color);
        g.fill(new RoundRectangle2D.Double(-length / 2, -width / 2, length, width, width / 2, width / 2));
        g.setColor(CAR_NOSE);
        g.fill(new Rectangle2D.Double(length / 2 - length / 5, -width / 2, length / 5, width));
        g.setTransform(saved);
    }

    private void drawScore(Graphics2D g) {
        g.setFont(new Font(Font.SANS_SERIF, Font.BOLD, 28));
        FontMetrics metrics = g.getFontMetrics();
        String blue = "BLUE " + state.score().blue();
        String orange = state.score().orange() + " ORANGE";
        int centre = getWidth() / 2;
        int baseline = SCORE_HEIGHT - 12;
        g.setColor(BLUE);
        g.drawString(blue, centre - 20 - metrics.stringWidth(blue), baseline);
        g.setColor(Color.WHITE);
        g.drawString("-", centre - metrics.stringWidth("-") / 2, baseline);
        g.setColor(ORANGE);
        g.drawString(orange, centre + 20, baseline);
    }

    private void drawHud(Graphics2D g) {
        g.setColor(Color.WHITE);
        g.setFont(new Font(Font.MONOSPACED, Font.PLAIN, 14));
        int baseline = getHeight() - HUD_HEIGHT / 2;
        BallState ball = state.ball();
        g.drawString(
                "ball height %5.0f  speed %6.0f  |  time %5.1fs"
                        .formatted(ball.position().z(), ball.velocity().length(), state.tick() * config.tickSeconds()),
                MARGIN,
                baseline);
        g.drawString(controlsHelp, MARGIN, baseline + 18);
    }

    private void drawGoalBanner(Graphics2D g, Team scorer) {
        g.setFont(new Font(Font.SANS_SERIF, Font.BOLD, 72));
        String text = scorer + " SCORES!";
        int x = (getWidth() - g.getFontMetrics().stringWidth(text)) / 2;
        int y = getHeight() / 2;
        g.setColor(new Color(0, 0, 0, 140));
        g.drawString(text, x + 4, y + 4);
        g.setColor(colorOf(scorer));
        g.drawString(text, x, y);
    }

    private static Color colorOf(Team team) {
        return switch (team) {
            case BLUE -> BLUE;
            case ORANGE -> ORANGE;
        };
    }

    /**
     * Maps world units (X right, Y up, Z height) to screen pixels, fitting the arena and both nets
     * in the panel.
     */
    private record Projection(double scale, double centreX, double centreY) {

        Projection(ArenaSpec arena, int panelWidth, int panelHeight) {
            this(
                    Math.min(
                            (panelWidth - 2.0 * MARGIN) / (2 * (arena.halfLength() + arena.goalDepth())),
                            (panelHeight - 2.0 * MARGIN - SCORE_HEIGHT - HUD_HEIGHT) / (2 * arena.halfWidth())),
                    panelWidth / 2.0,
                    SCORE_HEIGHT + (panelHeight - SCORE_HEIGHT - HUD_HEIGHT) / 2.0);
        }

        Point2D toScreen(double x, double y, double height) {
            return new Point2D.Double(centreX + x * scale, centreY - (y + height * HEIGHT_SHIFT) * scale);
        }

        /** Screen rectangle for the floor area between two world corners. */
        Rectangle2D rectangle(double minX, double maxY, double maxX, double minY) {
            Point2D topLeft = toScreen(minX, maxY, 0);
            Point2D bottomRight = toScreen(maxX, minY, 0);
            return new Rectangle2D.Double(
                    topLeft.getX(),
                    topLeft.getY(),
                    bottomRight.getX() - topLeft.getX(),
                    bottomRight.getY() - topLeft.getY());
        }
    }
}
