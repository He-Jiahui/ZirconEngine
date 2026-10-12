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
