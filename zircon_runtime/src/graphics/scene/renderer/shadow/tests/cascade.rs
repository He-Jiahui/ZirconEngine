use super::*;

fn assert_approx_eq(lhs: Real, rhs: Real) {
    assert!(
        (lhs - rhs).abs() <= 0.0001,
        "expected {lhs} to be close to {rhs}"
    );
}

#[test]
fn render_shadow_cascade_splits_blend_log_linear() {
    let linear_config = CascadeSplitConfig {
        cascade_count: 4,
        max_distance: 100.0,
        log_linear_lambda: 0.0,
        fade_fraction: 0.1,
    };
    let linear = compute_cascade_splits(&linear_config, 1.0);
    assert_approx_eq(linear[1], 25.75);
    assert_approx_eq(linear[2], 50.5);
    assert_approx_eq(linear[3], 75.25);

    let logarithmic_config = CascadeSplitConfig {
        log_linear_lambda: 1.0,
        ..linear_config
    };
    let logarithmic = compute_cascade_splits(&logarithmic_config, 1.0);
    assert_approx_eq(logarithmic[1], 100.0_f32.powf(0.25));
    assert_approx_eq(logarithmic[2], 10.0);
    assert_approx_eq(logarithmic[4], 100.0);
}

#[test]
fn render_shadow_cascade_ranges_are_monotonic_and_have_fade_bands() {
    let config = CascadeSplitConfig::default();
    let ranges = compute_cascade_ranges(&config, 0.1);

    assert_eq!(ranges.len(), MAX_SHADOW_CASCADES);
    for range in &ranges {
        assert!(range.near < range.far);
        assert!(range.fade_start >= range.near);
        assert!(range.fade_start <= range.far);
        assert!(range.fade_length >= 0.0);
    }
    for pair in ranges.windows(2) {
        assert!(pair[0].far <= pair[1].far);
        assert!(pair[0].near < pair[1].near);
    }
}

#[test]
fn render_shadow_cascade_snapping_quantizes_origin() {
    let snapped =
        snap_light_space_center_to_texel(Mat4::IDENTITY, Vec3::new(10.25, 4.75, 2.0), 1.0);

    assert_eq!(snapped, Vec3::new(10.0, 4.0, 2.0));
}

#[test]
fn render_shadow_cascade_view_projection_is_stable_under_half_texel_motion() {
    let bounds =
        CascadeShadowBounds::new(Vec3::new(10.2, -3.8, 0.0), 64.0).with_depth_range(0.1, 256.0);
    let moved_bounds = CascadeShadowBounds {
        center: Vec3::new(10.6, -3.4, 0.0),
        ..bounds
    };

    let first = snapped_cascade_view_projection(Mat4::IDENTITY, bounds, 128);
    let moved = snapped_cascade_view_projection(Mat4::IDENTITY, moved_bounds, 128);

    assert_eq!(first.to_cols_array(), moved.to_cols_array());
    assert_approx_eq(cascade_world_units_per_texel(64.0, 128), 1.0);
}

#[test]
fn render_shadow_cascade_bounds_follow_camera_slice_depth() {
    let camera = ViewportCameraSnapshot {
        transform: crate::core::math::Transform::looking_at(
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::Y,
        ),
        aspect_ratio: 1.0,
        z_near: 0.1,
        z_far: 100.0,
        ..ViewportCameraSnapshot::default()
    };
    let near_bounds = cascade_shadow_bounds_from_camera_slice(
        &camera,
        CascadeRange {
            index: 0,
            near: 0.1,
            far: 8.0,
            fade_start: 7.0,
            fade_length: 1.0,
        },
    );
    let far_bounds = cascade_shadow_bounds_from_camera_slice(
        &camera,
        CascadeRange {
            index: 1,
            near: 8.0,
            far: 40.0,
            fade_start: 36.0,
            fade_length: 4.0,
        },
    );

    assert!(far_bounds.radius > near_bounds.radius);
    assert!(
        far_bounds.center.distance(camera.transform.translation)
            > near_bounds.center.distance(camera.transform.translation)
    );
}
