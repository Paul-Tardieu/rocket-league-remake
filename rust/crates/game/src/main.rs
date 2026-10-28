//! Desktop 3D window: you (BLUE) against a striker bot (ORANGE), or bot vs bot with `--watch`.
//!
//! Options: `--watch`, `--speed=N` (simulation N times faster than real time),
//! `--camera=ball|car|broadcast`, `--screenshot=PATH --after=SECONDS` (save one frame after that
//! much simulated time, then quit; used to check visuals without watching).

mod camera;
mod controls;
mod hud;
mod render;
mod shapes;

use camera::{CameraMode, CameraRig};
use carball_bot::{Bot, StrikerBot};
use carball_sim::{GameEvent, GameState, PhysicsConfig, Team, World};
use hud::{HudInfo, draw_hud};
use macroquad::prelude::*;

const SKY: Color = Color::new(0.52, 0.66, 0.84, 1.0);
const GOAL_BANNER_SECONDS: f32 = 2.0;
/// Longest stretch of real time simulated in one frame, so a stall does not trigger a burst.
const MAX_FRAME_SECONDS: f64 = 0.25;

struct Options {
    watch: bool,
    speed: f64,
    camera: CameraMode,
    screenshot: Option<(String, f64)>,
}

impl Options {
    fn from_args() -> Self {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let value = |name: &str| {
            args.iter()
                .find_map(|arg| arg.strip_prefix(name).map(str::to_owned))
        };
        let speed = value("--speed=").map_or(1.0, |v| v.parse().expect("--speed must be a number"));
        assert!(speed > 0.0, "--speed must be positive");
        let camera = match value("--camera=").as_deref() {
            None | Some("ball") => CameraMode::Ball,
            Some("car") => CameraMode::Car,
            Some("broadcast") => CameraMode::Broadcast,
            Some(other) => panic!("unknown camera {other}, expected ball, car or broadcast"),
        };
        let after = value("--after=").map_or(3.0, |v| v.parse().expect("--after must be a number"));
        Self {
            watch: args.iter().any(|arg| arg == "--watch"),
            speed,
            camera,
            screenshot: value("--screenshot=").map(|path| (path, after)),
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Carball".to_owned(),
        window_width: 1280,
        window_height: 800,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let options = Options::from_args();
    let config = PhysicsConfig::default();
    let mut world = World::one_versus_one(config.clone());
    let teams = [Team::Blue, Team::Orange];
    // The keyboard is just another Bot: something that turns a GameState into a CarInput.
    let blue: Box<dyn Bot> = if options.watch {
        Box::new(StrikerBot::new(0, Team::Blue, config.clone(), 1))
    } else {
        Box::new(|_: &GameState| controls::read_player_input())
    };
    let mut controllers: Vec<Box<dyn Bot>> = vec![
        blue,
        Box::new(StrikerBot::new(1, Team::Orange, config.clone(), 2)),
    ];
    let help = if options.watch {
        "Bot contre bot   Espace: pause  C: camera  R: reset  Echap: quitter"
    } else {
        controls::HELP
    };

    let tick_seconds = config.tick_seconds();
    let mut camera_mode = if options.watch && options.camera == CameraMode::Ball {
        CameraMode::Broadcast
    } else {
        options.camera
    };
    let mut rig = CameraRig::new();
    let mut unsimulated = 0.0;
    let mut paused = false;
    let mut banner: Option<(Team, f32)> = None;

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }
        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::C) {
            camera_mode = camera_mode.next();
        }
        if is_key_pressed(KeyCode::R) {
            world.reset();
            banner = None;
        }

        let frame_seconds = get_frame_time();
        if !paused {
            unsimulated += f64::from(frame_seconds).min(MAX_FRAME_SECONDS) * options.speed;
            // Fixed timestep: real time is banked and spent in whole, identical simulation ticks.
            while unsimulated >= tick_seconds {
                let state = world.state();
                let inputs: Vec<_> = controllers.iter_mut().map(|bot| bot.act(&state)).collect();
                for event in world.step(&inputs) {
                    if let GameEvent::Goal { scorer } = event {
                        banner = Some((scorer, GOAL_BANNER_SECONDS));
                    }
                }
                unsimulated -= tick_seconds;
            }
            banner = banner
                .map(|(team, left)| (team, left - frame_seconds))
                .filter(|(_, left)| *left > 0.0);
        }

        let state = world.state();
        rig.update(camera_mode, &state, &config, 0, frame_seconds);
        clear_background(SKY);
        set_camera(&rig.camera());
        render::draw_scene(&state, &config, &teams, get_time() as f32);
        set_default_camera();
        draw_hud(&HudInfo {
            state: &state,
            config: &config,
            followed: 0,
            paused,
            goal_banner: banner.map(|(team, _)| team),
            speed: options.speed,
            camera_label: camera_mode.label(),
            help,
        });

        if let Some((path, after)) = &options.screenshot
            && state.tick as f64 * tick_seconds >= *after
        {
            get_screen_data().export_png(path);
            break;
        }
        next_frame().await;
    }
}
