package io.github.paultrde.carball.sim;

import static org.assertj.core.api.Assertions.assertThat;

import org.junit.jupiter.api.Test;

class Vec3Test {

    @Test
    void addsComponentWise() {
        assertThat(new Vec3(1, 2, 3).plus(new Vec3(4, 5, 6))).isEqualTo(new Vec3(5, 7, 9));
    }

    @Test
    void computesLength() {
        assertThat(new Vec3(3, 4, 0).length()).isEqualTo(5.0);
    }
}
