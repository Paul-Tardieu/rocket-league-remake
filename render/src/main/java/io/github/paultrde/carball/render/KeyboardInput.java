package io.github.paultrde.carball.render;

import io.github.paultrde.carball.sim.CarInput;
import java.awt.event.KeyAdapter;
import java.awt.event.KeyEvent;
import java.util.HashSet;
import java.util.Set;

/**
 * Turns the keys currently held down into a {@link CarInput}. Key events only record state; the
 * game loop reads it once per frame, so input is sampled at a fixed point in the loop.
 */
final class KeyboardInput extends KeyAdapter {

    private final Set<Integer> pressed = new HashSet<>();
    private boolean resetRequested;

    @Override
    public void keyPressed(KeyEvent e) {
        pressed.add(e.getKeyCode());
        if (e.getKeyCode() == KeyEvent.VK_R) {
            resetRequested = true;
        }
    }

    @Override
    public void keyReleased(KeyEvent e) {
        pressed.remove(e.getKeyCode());
    }

    CarInput current() {
        double throttle = axis(KeyEvent.VK_W, KeyEvent.VK_UP) - axis(KeyEvent.VK_S, KeyEvent.VK_DOWN);
        double steer = axis(KeyEvent.VK_A, KeyEvent.VK_LEFT) - axis(KeyEvent.VK_D, KeyEvent.VK_RIGHT);
        return new CarInput(throttle, steer, pressed.contains(KeyEvent.VK_SPACE));
    }

    /** Returns true once per R key press. */
    boolean consumeReset() {
        boolean requested = resetRequested;
        resetRequested = false;
        return requested;
    }

    private double axis(int key, int alternateKey) {
        return pressed.contains(key) || pressed.contains(alternateKey) ? 1 : 0;
    }
}
