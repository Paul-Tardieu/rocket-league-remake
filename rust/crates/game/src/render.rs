//! Draws the arena, ball, cars and boost pads in 3D. Read-only: it never touches the simulation.

use crate::shapes::{draw_ball, draw_block, draw_box, draw_disc, draw_floor_strip};
use carball_sim::{CarState, DQuat, DVec3, GameState, PhysicsConfig, Team};
use macroquad::prelude::{Color, Quat, Vec3, vec3};

pub const BLUE: Color = Color::new(0.16, 0.47, 0.95, 1.0);
pub const ORANGE: Color = Color::new(0.98, 0.52, 0.10, 1.0);
const TURF_A: Color = Color::new(0.17, 0.46, 0.22, 1.0);
const TURF_B: Color = Color::new(0.15, 0.41, 0.20, 1.0);
const MARKING: Color = Color::new(1.0, 1.0, 1.0, 0.55);
const WALL: Color = Color::new(0.75, 0.85, 1.0, 0.10);
const WALL_EDGE: Color = Color::new(0.85, 0.92, 1.0, 0.55);
const BALL: Color = Color::new(0.93, 0.94, 0.96, 1.0);
const SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.35);
const PAD_READY: Color = Color::new(1.0, 0.82, 0.15, 1.0);
const PAD_EMPTY: Color = Color::new(0.35, 0.35, 0.35, 1.0);
const TYRE: Color = Color::new(0.08, 0.08, 0.09, 1.0);
const GLASS: Color = Color::new(0.10, 0.12, 0.16, 1.0);
const FLAME_OUTER: Color = Color::new(1.0, 0.45, 0.05, 0.9);
const FLAME_INNER: Color = Color::new(1.0, 0.92, 0.45, 1.0);
const STRIPES: u32 = 16;
const SURROUNDINGS: Color = Color::new(0.20, 0.22, 0.26, 1.0);

pub fn team_color(team: Team) -> Color {
    match team {
        Team::Blue => BLUE,
        Team::Orange => ORANGE,
    }
}

pub fn to_vec3(v: DVec3) -> Vec3 {
    vec3(v.x as f32, v.y as f32, v.z as f32)
}

fn to_quat(q: DQuat) -> Quat {
    Quat::from_xyzw(q.x as f32, q.y as f32, q.z as f32, q.w as f32)
}

/// Draws one frame of the 3D scene. `time` only animates cosmetic effects (boost flame flicker).
pub fn draw_scene(state: &GameState, config: &PhysicsConfig, teams: &[Team], time: f32) {
    draw_floor(config);
    draw_markings(config);
    draw_pads(state, config, time);
    draw_shadows(state, config);
    for (car, team) in state.cars.iter().zip(teams) {
        draw_car(car, config, team_color(*team), time);
    }
    draw_ball(
        to_vec3(state.ball.position),
        config.ball.radius as f32,
        BALL,
    );
    // Translucent things last, so what is behind them has already been drawn.
    draw_goals(config);
    draw_walls(config);
}

fn draw_floor(config: &PhysicsConfig) {
    let arena = &config.arena;
    let (hl, hw) = (arena.half_length as f32, arena.half_width as f32);
    // Ground around the arena, slightly lower so it never fights with the pitch for depth.
    draw_block(
        vec3(0.0, 0.0, -40.0),
        vec3(hl * 4.0, hw * 4.0, 20.0),
        SURROUNDINGS,
    );
    let stripe = 2.0 * hl / STRIPES as f32;
    for i in 0..STRIPES {
        let x = -hl + stripe * (i as f32 + 0.5);
        let color = if i % 2 == 0 { TURF_A } else { TURF_B };
        draw_block(vec3(x, 0.0, -10.0), vec3(stripe / 2.0, hw, 10.0), color);
    }
    // Floor inside both nets.
    let depth = arena.goal_depth as f32;
    for side in [-1.0, 1.0] {
        draw_block(
            vec3(side * (hl + depth / 2.0), 0.0, -10.0),
            vec3(depth / 2.0, arena.goal_half_width as f32, 10.0),
            TURF_B,
        );
    }
}

fn draw_markings(config: &PhysicsConfig) {
    let arena = &config.arena;
    let (hl, hw, z) = (arena.half_length as f32, arena.half_width as f32, 1.0);
    let width = 18.0;
    draw_floor_strip(vec3(0.0, -hw, z), vec3(0.0, hw, z), width, MARKING);
    let radius = 1000.0;
    let segments = 48;
    for i in 0..segments {
        let a = std::f32::consts::TAU * i as f32 / segments as f32;
        let b = std::f32::consts::TAU * (i + 1) as f32 / segments as f32;
        draw_floor_strip(
            vec3(a.cos() * radius, a.sin() * radius, z),
            vec3(b.cos() * radius, b.sin() * radius, z),
            width,
            MARKING,
        );
    }
    // Penalty-style boxes in front of each goal.
    let (box_depth, box_half_width) = (1100.0, arena.goal_half_width as f32 + 500.0);
    for side in [-1.0f32, 1.0] {
        let line = side * hl;
        let front = side * (hl - box_depth);
        draw_floor_strip(
            vec3(line, box_half_width, z),
            vec3(front, box_half_width, z),
            width,
            MARKING,
        );
        draw_floor_strip(
            vec3(front, box_half_width, z),
            vec3(front, -box_half_width, z),
            width,
            MARKING,
        );
        draw_floor_strip(
            vec3(front, -box_half_width, z),
            vec3(line, -box_half_width, z),
            width,
            MARKING,
        );
        draw_floor_strip(vec3(line, hw, z), vec3(line, -hw, z), width, MARKING);
    }
}

fn draw_pads(state: &GameState, config: &PhysicsConfig, time: f32) {
    for (pad, respawn) in config.pads.iter().zip(&state.pad_respawn) {
        let ready = *respawn <= 0.0;
        let color = if ready { PAD_READY } else { PAD_EMPTY };
        let base = to_vec3(pad.position) + vec3(0.0, 0.0, 2.0);
        let radius = if pad.big { 110.0 } else { 60.0 };
        draw_disc(base, radius, color);
        if ready && pad.big {
            let bob = (time * 2.5).sin() * 12.0;
            draw_ball(base + vec3(0.0, 0.0, 85.0 + bob), 38.0, PAD_READY);
        }
    }
}

fn draw_shadows(state: &GameState, config: &PhysicsConfig) {
    let ball = to_vec3(state.ball.position);
    let radius = config.ball.radius as f32;
    // Higher ball, smaller shadow: a cheap but strong depth cue.
    let scale = (1.0 - (ball.z - radius) / 2500.0).clamp(0.35, 1.0);
    draw_disc(vec3(ball.x, ball.y, 3.0), radius * scale, SHADOW);
    for car in &state.cars {
        let p = to_vec3(car.position);
        draw_disc(vec3(p.x, p.y, 3.0), 70.0, SHADOW);
    }
}

/// Car body in car space: X forward, Y left, Z up, origin at the hitbox centre (60 above floor).
fn draw_car(car: &CarState, config: &PhysicsConfig, paint: Color, time: f32) {
    let rotation = to_quat(car.orientation);
    let origin = to_vec3(car.position);
    let floor = -(config.car.hitbox_radius as f32);
    let part = |local: Vec3, half: Vec3, color: Color| {
        draw_box(origin + rotation * local, rotation, half, color)
    };

    part(vec3(0.0, 0.0, floor + 32.0), vec3(64.0, 40.0, 14.0), paint);
    part(vec3(-12.0, 0.0, floor + 54.0), vec3(30.0, 33.0, 9.0), GLASS);
    part(vec3(-60.0, 0.0, floor + 52.0), vec3(5.0, 38.0, 3.0), paint);
    for (x, y) in [(40.0, 41.0), (40.0, -41.0), (-38.0, 41.0), (-38.0, -41.0)] {
        part(vec3(x, y, floor + 14.0), vec3(14.0, 8.0, 14.0), TYRE);
    }
    if car.boosting {
        let flicker = 0.75 + 0.25 * (time * 60.0).sin();
        part(
            vec3(-82.0 - 12.0 * flicker, 0.0, floor + 32.0),
            vec3(18.0 * flicker, 11.0, 8.0),
            FLAME_OUTER,
        );
        part(
            vec3(-74.0, 0.0, floor + 32.0),
            vec3(10.0, 6.0, 5.0),
            FLAME_INNER,
        );
    }
}

fn draw_goals(config: &PhysicsConfig) {
    let arena = &config.arena;
    let (hl, depth) = (arena.half_length as f32, arena.goal_depth as f32);
    let (gw, gh) = (arena.goal_half_width as f32, arena.goal_height as f32);
    let post = 14.0;
    // BLUE defends -X, ORANGE defends +X.
    for (side, team) in [(-1.0f32, Team::Blue), (1.0, Team::Orange)] {
        let color = team_color(team);
        let net = Color::new(color.r, color.g, color.b, 0.18);
        draw_block(
            vec3(side * (hl + depth / 2.0), 0.0, gh / 2.0),
            vec3(depth / 2.0, gw, gh / 2.0),
            net,
        );
        for y in [-gw, gw] {
            draw_block(
                vec3(side * hl, y, gh / 2.0),
                vec3(post, post, gh / 2.0),
                color,
            );
        }
        draw_block(vec3(side * hl, 0.0, gh), vec3(post, gw + post, post), color);
    }
}

fn draw_walls(config: &PhysicsConfig) {
    let arena = &config.arena;
    let (hl, hw, top) = (
        arena.half_length as f32,
        arena.half_width as f32,
        arena.ceiling as f32,
    );
    let (gw, gh) = (arena.goal_half_width as f32, arena.goal_height as f32);
    let thin = 4.0;
    for y in [-hw, hw] {
        draw_block(vec3(0.0, y, top / 2.0), vec3(hl, thin, top / 2.0), WALL);
    }
    for x in [-hl, hl] {
        // End wall around the goal mouth: two side panels and a lintel above the goal.
        let panel = (hw - gw) / 2.0;
        for y in [-(gw + panel), gw + panel] {
            draw_block(vec3(x, y, top / 2.0), vec3(thin, panel, top / 2.0), WALL);
        }
        draw_block(
            vec3(x, 0.0, (gh + top) / 2.0),
            vec3(thin, gw, (top - gh) / 2.0),
            WALL,
        );
    }
    // Bright top edges so the arena reads as a box.
    for (a, b) in [
        (vec3(-hl, -hw, top), vec3(hl, -hw, top)),
        (vec3(-hl, hw, top), vec3(hl, hw, top)),
        (vec3(-hl, -hw, top), vec3(-hl, hw, top)),
        (vec3(hl, -hw, top), vec3(hl, hw, top)),
    ] {
        macroquad::models::draw_line_3d(a, b, WALL_EDGE);
    }
    for (x, y) in [(-hl, -hw), (-hl, hw), (hl, -hw), (hl, hw)] {
        macroquad::models::draw_line_3d(vec3(x, y, 0.0), vec3(x, y, top), WALL_EDGE);
    }
}
