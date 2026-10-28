//! Shaded primitives built as meshes. macroquad's own 3D shapes are flat-coloured and assume Y is
//! up; these bake a simple directional light into vertex colours and work in our Z-up world.

use macroquad::models::{Mesh, Vertex, draw_mesh};
use macroquad::prelude::{Color, Quat, Vec3, vec3};

/// Direction the light comes from (normalised in [`shade`]).
const LIGHT: Vec3 = Vec3::new(0.35, 0.55, 0.85);
const AMBIENT: f32 = 0.45;

fn shade(color: Color, normal: Vec3) -> Color {
    let light = AMBIENT + (1.0 - AMBIENT) * normal.dot(LIGHT.normalize()).max(0.0);
    Color::new(color.r * light, color.g * light, color.b * light, color.a)
}

fn vertex(position: Vec3, color: Color) -> Vertex {
    Vertex::new(position.x, position.y, position.z, 0.0, 0.0, color)
}

/// A box rotated by `rotation` around `center`, with half sizes `half` along its local axes.
pub fn draw_box(center: Vec3, rotation: Quat, half: Vec3, color: Color) {
    const FACES: [(Vec3, Vec3, Vec3); 6] = [
        (Vec3::X, Vec3::Y, Vec3::Z),
        (Vec3::NEG_X, Vec3::Z, Vec3::Y),
        (Vec3::Y, Vec3::Z, Vec3::X),
        (Vec3::NEG_Y, Vec3::X, Vec3::Z),
        (Vec3::Z, Vec3::X, Vec3::Y),
        (Vec3::NEG_Z, Vec3::Y, Vec3::X),
    ];
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);
    for (normal, u, v) in FACES {
        let face_color = shade(color, rotation * normal);
        let base = vertices.len() as u16;
        for (su, sv) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let local = (normal + u * su + v * sv) * half;
            vertices.push(vertex(center + rotation * local, face_color));
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

/// An axis-aligned box from its centre and half sizes.
pub fn draw_block(center: Vec3, half: Vec3, color: Color) {
    draw_box(center, Quat::IDENTITY, half, color);
}

/// A sphere with baked lighting.
pub fn draw_ball(center: Vec3, radius: f32, color: Color) {
    const RINGS: u16 = 14;
    const SECTORS: u16 = 28;
    let mut vertices = Vec::with_capacity(usize::from((RINGS + 1) * (SECTORS + 1)));
    for ring in 0..=RINGS {
        let polar = std::f32::consts::PI * f32::from(ring) / f32::from(RINGS);
        for sector in 0..=SECTORS {
            let azimuth = std::f32::consts::TAU * f32::from(sector) / f32::from(SECTORS);
            let normal = vec3(
                polar.sin() * azimuth.cos(),
                polar.sin() * azimuth.sin(),
                polar.cos(),
            );
            vertices.push(vertex(center + normal * radius, shade(color, normal)));
        }
    }
    let mut indices = Vec::with_capacity(usize::from(RINGS * SECTORS * 6));
    for ring in 0..RINGS {
        for sector in 0..SECTORS {
            let a = ring * (SECTORS + 1) + sector;
            let b = a + SECTORS + 1;
            indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

/// A flat disc lying on a horizontal plane at `center.z`.
pub fn draw_disc(center: Vec3, radius: f32, color: Color) {
    const SEGMENTS: u16 = 32;
    let mut vertices = vec![vertex(center, color)];
    for i in 0..=SEGMENTS {
        let angle = std::f32::consts::TAU * f32::from(i) / f32::from(SEGMENTS);
        vertices.push(vertex(
            center + vec3(angle.cos(), angle.sin(), 0.0) * radius,
            color,
        ));
    }
    let indices = (1..=SEGMENTS).flat_map(|i| [0, i, i + 1]).collect();
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

/// A flat strip on the floor from `a` to `b`, for field markings that stay visible from afar.
pub fn draw_floor_strip(a: Vec3, b: Vec3, width: f32, color: Color) {
    let along = (b - a).normalize_or_zero();
    let side = vec3(-along.y, along.x, 0.0) * (width / 2.0);
    let vertices = vec![
        vertex(a - side, color),
        vertex(a + side, color),
        vertex(b + side, color),
        vertex(b - side, color),
    ];
    draw_mesh(&Mesh {
        vertices,
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    });
}
