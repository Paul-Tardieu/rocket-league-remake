package io.github.paultrde.carball.sim;

import static org.assertj.core.api.Assertions.assertThatIllegalArgumentException;

import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

class CarInputTest {

    @ParameterizedTest
    @ValueSource(doubles = {-1.01, 1.01, Double.NaN})
    void rejectsThrottleOutsideRange(double throttle) {
        assertThatIllegalArgumentException().isThrownBy(() -> new CarInput(throttle, 0, false));
    }
}
