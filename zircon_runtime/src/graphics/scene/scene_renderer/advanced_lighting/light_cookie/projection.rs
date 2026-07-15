use crate::core::math::{Vec2, Vec3};

const PROJECTION_EPSILON: f32 = 1.0e-6;

pub(crate) fn directional_cookie_uv(
    world_position: Vec3,
    light_direction: Vec3,
    offset: Vec2,
    scale: Vec2,
) -> Vec2 {
    let (right, up, _) = light_basis(light_direction);
    Vec2::new(world_position.dot(right), world_position.dot(up)) * scale + offset
}

pub(crate) fn spot_cookie_uv(
    world_position: Vec3,
    light_position: Vec3,
    light_direction: Vec3,
    outer_angle_radians: f32,
) -> Option<Vec2> {
    let (right, up, forward) = light_basis(light_direction);
    let local = world_position - light_position;
    let depth = local.dot(forward);
    let half_extent = depth * outer_angle_radians.tan();
    if depth <= PROJECTION_EPSILON || half_extent.abs() <= PROJECTION_EPSILON {
        return None;
    }
    Some(Vec2::new(local.dot(right), local.dot(up)) / (2.0 * half_extent) + Vec2::splat(0.5))
}

pub(crate) fn point_octahedral_cookie_uv(direction: Vec3) -> Vec2 {
    let denominator = direction.x.abs() + direction.y.abs() + direction.z.abs();
    if denominator <= PROJECTION_EPSILON {
        return Vec2::splat(0.5);
    }
    let normalized = direction / denominator;
    let mut folded = normalized.truncate();
    if normalized.z < 0.0 {
        folded = Vec2::new(
            (1.0 - normalized.y.abs()) * normalized.x.signum(),
            (1.0 - normalized.x.abs()) * normalized.y.signum(),
        );
    }
    folded * 0.5 + Vec2::splat(0.5)
}

fn light_basis(direction: Vec3) -> (Vec3, Vec3, Vec3) {
    let forward = direction.try_normalize().unwrap_or(Vec3::NEG_Z);
    let reference_up = if forward.dot(Vec3::Y).abs() < 0.999 {
        Vec3::Y
    } else {
        Vec3::X
    };
    let right = forward.cross(reference_up).normalize();
    let up = right.cross(forward).normalize();
    (right, up, forward)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1.0e-5;

    #[test]
    fn render_cookie_uv_three_projections_match_reference() {
        let directional = directional_cookie_uv(
            Vec3::new(2.0, 3.0, 4.0),
            Vec3::NEG_Z,
            Vec2::new(0.25, 0.5),
            Vec2::new(0.5, 0.25),
        );
        assert_vec2_near(directional, Vec2::new(1.25, 1.25));

        let spot = spot_cookie_uv(
            Vec3::new(1.0, 0.0, -2.0),
            Vec3::ZERO,
            Vec3::NEG_Z,
            45.0_f32.to_radians(),
        )
        .expect("point lies in front of the spot light");
        assert_vec2_near(spot, Vec2::new(0.75, 0.5));

        assert_vec2_near(point_octahedral_cookie_uv(Vec3::X), Vec2::new(1.0, 0.5));
        assert_vec2_near(point_octahedral_cookie_uv(Vec3::NEG_Z), Vec2::new(1.0, 1.0));
    }

    fn assert_vec2_near(actual: Vec2, expected: Vec2) {
        assert!(
            (actual.x - expected.x).abs() <= EPSILON,
            "x: {actual:?} != {expected:?}"
        );
        assert!(
            (actual.y - expected.y).abs() <= EPSILON,
            "y: {actual:?} != {expected:?}"
        );
    }
}
