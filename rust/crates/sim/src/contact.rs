use glam::DVec3;

/// A sphere taking part in a collision.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Body {
    pub position: DVec3,
    pub velocity: DVec3,
    pub radius: f64,
    pub mass: f64,
}

/// Position and velocity changes to apply to each body of a contact.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Contact {
    pub push_a: DVec3,
    pub kick_a: DVec3,
    pub push_b: DVec3,
    pub kick_b: DVec3,
}

/// Sphere-vs-sphere contact resolved with an impulse. Returns `None` if the spheres don't touch.
///
/// 1. The overlap is removed by pushing the spheres apart; the lighter one moves more.
/// 2. If they still approach each other, an impulse along the contact normal exchanges momentum:
///    `j = -(1 + e)·v_rel / (1/m_a + 1/m_b)`, which conserves total momentum.
pub(crate) fn sphere_contact(a: Body, b: Body, restitution: f64) -> Option<Contact> {
    let delta = b.position - a.position;
    let distance = delta.length();
    let penetration = a.radius + b.radius - distance;
    if penetration <= 0.0 || distance == 0.0 {
        return None;
    }

    let normal = delta / distance;
    let (inverse_a, inverse_b) = (1.0 / a.mass, 1.0 / b.mass);
    let inverse_sum = inverse_a + inverse_b;

    let correction = normal * (penetration / inverse_sum);
    let mut contact = Contact {
        push_a: -correction * inverse_a,
        push_b: correction * inverse_b,
        ..Contact::default()
    };

    let approach_speed = (b.velocity - a.velocity).dot(normal);
    if approach_speed < 0.0 {
        let impulse = normal * (-(1.0 + restitution) * approach_speed / inverse_sum);
        contact.kick_a = -impulse * inverse_a;
        contact.kick_b = impulse * inverse_b;
    }
    Some(contact)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(x: f64, vx: f64, radius: f64, mass: f64) -> Body {
        Body {
            position: DVec3::new(x, 0.0, 0.0),
            velocity: DVec3::new(vx, 0.0, 0.0),
            radius,
            mass,
        }
    }

    #[test]
    fn ignores_spheres_that_do_not_touch() {
        assert_eq!(
            sphere_contact(
                body(0.0, 1000.0, 60.0, 180.0),
                body(500.0, 0.0, 92.0, 30.0),
                0.6
            ),
            None
        );
    }

    #[test]
    fn conserves_momentum_and_launches_the_light_sphere() {
        let car = body(0.0, 1000.0, 60.0, 180.0);
        let ball = body(140.0, 0.0, 92.0, 30.0);

        let contact = sphere_contact(car, ball, 0.6).expect("touching");

        let before = car.velocity * car.mass + ball.velocity * ball.mass;
        let after = (car.velocity + contact.kick_a) * car.mass
            + (ball.velocity + contact.kick_b) * ball.mass;
        assert!((after - before).length() < 1e-6);
        assert!(contact.kick_b.x > 1000.0);
    }

    #[test]
    fn separates_overlapping_spheres() {
        let car = body(0.0, 0.0, 60.0, 180.0);
        let ball = body(100.0, 0.0, 92.0, 30.0);

        let contact = sphere_contact(car, ball, 0.6).expect("touching");

        let distance = (ball.position + contact.push_b - car.position - contact.push_a).length();
        assert!((distance - 152.0).abs() < 1e-9);
    }
}
