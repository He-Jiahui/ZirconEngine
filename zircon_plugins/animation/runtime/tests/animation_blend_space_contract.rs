use zircon_plugin_animation_runtime::{
    BlendSpace1D, BlendSpace2D, BlendSpacePoint1D, BlendSpacePoint2D,
};
use zircon_runtime::core::math::Vec2;

#[test]
fn blend_space_1d_interpolates_sorted_samples() {
    let blend = BlendSpace1D::compile([
        BlendSpacePoint1D::new(1.0, 30),
        BlendSpacePoint1D::new(-1.0, 10),
        BlendSpacePoint1D::new(0.0, 20),
    ])
    .unwrap();
    assert_eq!(
        blend.sample(-2.0).unwrap().as_pairs(),
        [(10, 1.0), (10, 0.0)]
    );
    assert_eq!(
        blend.sample(0.25).unwrap().as_pairs(),
        [(20, 0.75), (30, 0.25)]
    );
    assert_eq!(
        blend.sample(2.0).unwrap().as_pairs(),
        [(30, 1.0), (30, 0.0)]
    );
}

#[test]
fn blend_space_2d_triangulation_weights_sum_to_one() {
    let blend = BlendSpace2D::compile([
        BlendSpacePoint2D::new(Vec2::new(1.0, 1.0), 30),
        BlendSpacePoint2D::new(Vec2::new(0.0, 0.0), 10),
        BlendSpacePoint2D::new(Vec2::new(1.0, 0.0), 20),
        BlendSpacePoint2D::new(Vec2::new(0.0, 1.0), 40),
    ])
    .unwrap();
    let weights = blend.sample(Vec2::new(0.25, 0.25)).unwrap();
    assert!((weights.weight_sum() - 1.0).abs() <= 1.0e-6);
    assert!(weights.as_pairs().iter().all(|(_, weight)| *weight >= 0.0));
    assert_eq!(blend.triangle_count(), 2);
}

#[test]
fn blend_space_2d_projects_outside_sample_to_hull() {
    let blend = BlendSpace2D::compile([
        BlendSpacePoint2D::new(Vec2::new(0.0, 0.0), 10),
        BlendSpacePoint2D::new(Vec2::new(1.0, 0.0), 20),
        BlendSpacePoint2D::new(Vec2::new(0.0, 1.0), 30),
    ])
    .unwrap();
    let weights = blend.sample(Vec2::new(2.0, 2.0)).unwrap();
    assert!((weights.weight_sum() - 1.0).abs() <= 1.0e-6);
    assert!(weights.as_pairs().iter().all(|(_, weight)| *weight >= 0.0));
}

#[test]
fn blend_space_square_builds_two_non_overlapping_triangles() {
    let blend = BlendSpace2D::compile([
        BlendSpacePoint2D::new(Vec2::new(1.0, 1.0), 30),
        BlendSpacePoint2D::new(Vec2::new(0.0, 0.0), 10),
        BlendSpacePoint2D::new(Vec2::new(1.0, 0.0), 20),
        BlendSpacePoint2D::new(Vec2::new(0.0, 1.0), 40),
    ])
    .unwrap();
    assert_eq!(blend.triangle_count(), 2);
}
