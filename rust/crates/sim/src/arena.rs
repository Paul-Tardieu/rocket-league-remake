use crate::config::ArenaSpec;
use glam::DVec3;

const EPSILON: f64 = 1e-9;

/// Result of keeping a sphere inside the arena. Callers decide how to react to each wall hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Confined {
    pub position: DVec3,
    pub hit_end_wall: bool,
    pub hit_side_wall: bool,
    pub hit_ceiling: bool,
}

/// Keeps a sphere inside the arena, including inside a goal net once it went through the mouth.
///
/// `previous` is the position at the start of the tick: it tells whether the sphere was already in
/// a net, in which case the net walls apply instead of the end wall.
pub(crate) fn confine(arena: &ArenaSpec, previous: DVec3, next: DVec3, radius: f64) -> Confined {
    let DVec3 {
        mut x,
        mut y,
        mut z,
    } = next;
    let (mut hit_end_wall, mut hit_side_wall, mut hit_ceiling) = (false, false, false);

    let goal_line = arena.half_length - radius;
    if x.abs() > goal_line {
        if is_inside_goal(arena, previous, radius) || fits_through_goal_mouth(arena, next, radius) {
            let back_wall = arena.half_length + arena.goal_depth - radius;
            if x.abs() > back_wall {
                x = back_wall.copysign(x);
                hit_end_wall = true;
            }
            let net_side = arena.goal_half_width - radius;
            if y.abs() > net_side {
                y = net_side.copysign(y);
                hit_side_wall = true;
            }
            let roof = arena.goal_height - radius;
            if z > roof {
                z = roof;
                hit_ceiling = true;
            }
        } else {
            x = goal_line.copysign(x);
            hit_end_wall = true;
        }
    }

    let side_wall = arena.half_width - radius;
    if y.abs() > side_wall {
        y = side_wall.copysign(y);
        hit_side_wall = true;
    }
    let ceiling = arena.ceiling - radius;
    if z > ceiling {
        z = ceiling;
        hit_ceiling = true;
    }
    Confined {
        position: DVec3::new(x, y, z),
        hit_end_wall,
        hit_side_wall,
        hit_ceiling,
    }
}

fn is_inside_goal(arena: &ArenaSpec, position: DVec3, radius: f64) -> bool {
    position.x.abs() > arena.half_length - radius
        && fits_through_goal_mouth(arena, position, radius)
}

fn fits_through_goal_mouth(arena: &ArenaSpec, position: DVec3, radius: f64) -> bool {
    position.y.abs() <= arena.goal_half_width - radius + EPSILON
        && position.z <= arena.goal_height - radius + EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARENA: ArenaSpec = ArenaSpec::DEFAULT;
    const R: f64 = 50.0;

    #[test]
    fn lets_a_sphere_through_the_goal_mouth() {
        let before = DVec3::new(ARENA.half_length - 100.0, 0.0, R);
        let after = DVec3::new(ARENA.half_length + 100.0, 0.0, R);

        let confined = confine(&ARENA, before, after, R);

        assert_eq!(confined.position, after);
        assert!(!confined.hit_end_wall);
    }

    #[test]
    fn blocks_a_sphere_beside_the_goal() {
        let before = DVec3::new(ARENA.half_length - 100.0, ARENA.goal_half_width + 200.0, R);
        let after = DVec3::new(ARENA.half_length + 100.0, before.y, R);

        let confined = confine(&ARENA, before, after, R);

        assert_eq!(confined.position.x, ARENA.half_length - R);
        assert!(confined.hit_end_wall);
    }

    #[test]
    fn keeps_a_sphere_inside_the_net_sides() {
        let in_net = DVec3::new(ARENA.half_length + 300.0, 0.0, R);
        let sliding = DVec3::new(in_net.x, ARENA.goal_half_width + 100.0, R);

        let confined = confine(&ARENA, in_net, sliding, R);

        assert_eq!(confined.position.y, ARENA.goal_half_width - R);
        assert!(confined.hit_side_wall);
    }

    #[test]
    fn stops_at_the_ceiling() {
        let confined = confine(
            &ARENA,
            DVec3::new(0.0, 0.0, 1000.0),
            DVec3::new(0.0, 0.0, 5000.0),
            R,
        );

        assert_eq!(confined.position.z, ARENA.ceiling - R);
        assert!(confined.hit_ceiling);
    }
}
