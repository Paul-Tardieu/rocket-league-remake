package io.github.paultrde.carball.sim;

import static org.assertj.core.api.Assertions.assertThat;

import org.junit.jupiter.api.Test;

class ArenaBoundsTest {

    private static final ArenaSpec ARENA = ArenaSpec.DEFAULT;
    private static final double RADIUS = 50;

    private final ArenaBounds bounds = new ArenaBounds(ARENA);

    @Test
    void letsASphereThroughTheGoalMouth() {
        Vec3 before = new Vec3(ARENA.halfLength() - 100, 0, RADIUS);
        Vec3 after = new Vec3(ARENA.halfLength() + 100, 0, RADIUS);

        ArenaBounds.Confined confined = bounds.confine(before, after, RADIUS);

        assertThat(confined.position()).isEqualTo(after);
        assertThat(confined.hitEndWall()).isFalse();
    }

    @Test
    void blocksASphereBesideTheGoal() {
        Vec3 before = new Vec3(ARENA.halfLength() - 100, ARENA.goalHalfWidth() + 200, RADIUS);
        Vec3 after = before.withX(ARENA.halfLength() + 100);

        ArenaBounds.Confined confined = bounds.confine(before, after, RADIUS);

        assertThat(confined.position().x()).isEqualTo(ARENA.halfLength() - RADIUS);
        assertThat(confined.hitEndWall()).isTrue();
    }

    @Test
    void keepsASphereInsideTheNetSides() {
        Vec3 inNet = new Vec3(ARENA.halfLength() + 300, 0, RADIUS);
        Vec3 slidingSideways = inNet.withY(ARENA.goalHalfWidth() + 100);

        ArenaBounds.Confined confined = bounds.confine(inNet, slidingSideways, RADIUS);

        assertThat(confined.position().y()).isEqualTo(ARENA.goalHalfWidth() - RADIUS);
        assertThat(confined.hitSideWall()).isTrue();
    }
}
