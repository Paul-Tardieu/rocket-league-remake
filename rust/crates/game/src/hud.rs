//! 2D overlay drawn on top of the 3D scene: score, clock, boost, speed, pause and goal banners.

use crate::render::{BLUE, ORANGE, team_color};
use carball_sim::{GameState, PhysicsConfig, Team};
use macroquad::prelude::{
    Color, WHITE, draw_rectangle, draw_rectangle_lines, draw_text, measure_text, screen_height,
    screen_width,
};

const PANEL: Color = Color::new(0.05, 0.06, 0.09, 0.75);

pub struct HudInfo<'a> {
    pub state: &'a GameState,
    pub config: &'a PhysicsConfig,
    /// Car whose boost and speed are shown.
    pub followed: usize,
    pub paused: bool,
    pub goal_banner: Option<Team>,
    pub speed: f64,
    pub camera_label: &'a str,
    pub help: &'a str,
}

pub fn draw_hud(info: &HudInfo) {
    draw_scoreboard(info);
    draw_car_gauges(info);
    draw_text(
        info.help,
        16.0,
        screen_height() - 14.0,
        18.0,
        Color::new(1.0, 1.0, 1.0, 0.7),
    );
    let mut status = format!("camera: {}", info.camera_label);
    if (info.speed - 1.0).abs() > f64::EPSILON {
        status.push_str(&format!("   vitesse x{}", info.speed));
    }
    draw_text(&status, 16.0, 28.0, 20.0, WHITE);
    if let Some(team) = info.goal_banner {
        let text = match team {
            Team::Blue => "BUT BLEU !",
            Team::Orange => "BUT ORANGE !",
        };
        centered_text(text, screen_height() * 0.38, 90.0, team_color(team));
    }
    if info.paused {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.45),
        );
        centered_text("PAUSE", screen_height() * 0.5, 110.0, WHITE);
        centered_text(
            "Espace pour reprendre",
            screen_height() * 0.5 + 50.0,
            30.0,
            WHITE,
        );
    }
}

fn draw_scoreboard(info: &HudInfo) {
    let score = info.state.score;
    let seconds = info.state.tick / u64::from(info.config.ticks_per_second);
    let (box_w, box_h, clock_w) = (90.0, 54.0, 120.0);
    let left = screen_width() / 2.0 - clock_w / 2.0 - box_w;
    draw_rectangle(left, 10.0, box_w, box_h, BLUE);
    draw_rectangle(left + box_w, 10.0, clock_w, box_h, PANEL);
    draw_rectangle(left + box_w + clock_w, 10.0, box_w, box_h, ORANGE);
    text_in_box(&score.blue.to_string(), left, box_w, 50.0);
    text_in_box(
        &format!("{}:{:02}", seconds / 60, seconds % 60),
        left + box_w,
        clock_w,
        36.0,
    );
    text_in_box(
        &score.orange.to_string(),
        left + box_w + clock_w,
        box_w,
        50.0,
    );
}

fn draw_car_gauges(info: &HudInfo) {
    let Some(car) = info.state.cars.get(info.followed) else {
        return;
    };
    let (w, h) = (260.0, 26.0);
    let x = screen_width() - w - 24.0;
    let y = screen_height() - 90.0;
    draw_rectangle(x - 10.0, y - 44.0, w + 20.0, 90.0, PANEL);
    let fill = (car.boost / 100.0) as f32;
    draw_rectangle(x, y, w * fill, h, Color::new(1.0, 0.65, 0.1, 1.0));
    draw_rectangle_lines(x, y, w, h, 2.0, WHITE);
    draw_text(format!("BOOST {:.0}", car.boost), x, y - 12.0, 30.0, WHITE);
    let speed = car.velocity.length();
    let at_max = speed >= info.config.car.max_speed - 1.0;
    let speed_color = if at_max {
        Color::new(1.0, 0.85, 0.2, 1.0)
    } else {
        WHITE
    };
    draw_text(
        format!("{speed:4.0} uu/s"),
        x + 150.0,
        y - 12.0,
        24.0,
        speed_color,
    );
}

fn text_in_box(text: &str, x: f32, width: f32, size: f32) {
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(
        text,
        x + (width - dims.width) / 2.0,
        10.0 + 27.0 + dims.height / 2.0,
        size,
        WHITE,
    );
}

fn centered_text(text: &str, y: f32, size: f32, color: Color) {
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(
        text,
        (screen_width() - dims.width) / 2.0 + 3.0,
        y + 3.0,
        size,
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    draw_text(text, (screen_width() - dims.width) / 2.0, y, size, color);
}
