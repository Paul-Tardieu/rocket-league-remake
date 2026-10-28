//! Keyboard and mouse to [`CarInput`]. Both AZERTY (ZQSD) and QWERTY (WASD) layouts work, as do
//! the arrow keys. Movement keys double as air control: forward tips the nose down, so jump +
//! forward + jump is a front flip.

use carball_sim::CarInput;
use macroquad::prelude::{KeyCode, MouseButton, is_key_down, is_mouse_button_down};

fn any_down(keys: &[KeyCode]) -> f64 {
    if keys.iter().any(|&key| is_key_down(key)) {
        1.0
    } else {
        0.0
    }
}

pub fn read_player_input() -> CarInput {
    let throttle =
        any_down(&[KeyCode::Up, KeyCode::W, KeyCode::Z]) - any_down(&[KeyCode::Down, KeyCode::S]);
    let steer = any_down(&[KeyCode::Left, KeyCode::A, KeyCode::Q])
        - any_down(&[KeyCode::Right, KeyCode::D]);
    let roll = any_down(&[KeyCode::E]) - any_down(&[KeyCode::LeftAlt]);
    CarInput {
        throttle,
        steer,
        pitch: throttle,
        yaw: steer,
        roll,
        jump: is_key_down(KeyCode::J) || is_mouse_button_down(MouseButton::Right),
        boost: is_key_down(KeyCode::K)
            || is_key_down(KeyCode::LeftShift)
            || is_mouse_button_down(MouseButton::Left),
    }
}

pub const HELP: &str = "ZQSD/WASD/fleches: conduire  J/clic droit: sauter (2x = flip)  K/Maj/clic gauche: boost  \
                        Espace: pause  C: camera  R: reset  Echap: quitter";
