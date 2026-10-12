use super::*;

#[test]
fn depth_of_field_prepare_params_sanitize_camera_and_lens_values() {
    let camera = ViewportCameraSnapshot {
        z_near: 0.25,
        z_far: 64.0,
        projection_mode: ProjectionMode::Perspective,
        ..Default::default()
    };
    let params = DepthOfFieldPrepareParams::from_camera(
        UVec2::new(1920, 1080),
        [0, 0],
        &camera,
        RenderDepthOfFieldSettings {
            focus_distance: 8.0,
            focus_range: -1.0,
            aperture: 0.75,
            focal_length_mm: 400.0,
            max_blur_radius: 6.0,
            ..Default::default()
        },
    );

    assert_eq!(params.viewport, [1920, 1080, 0, 0]);
    assert_near(params.depth[0], 0.25);
    assert_near(params.depth[1], 64.0);
    assert_near(params.depth[2], 1.0 / 63.75);
    assert_near(params.depth[3], 1.0);
    assert_near(params.lens[0], 8.0);
    assert_near(params.lens[1], 0.001);
    assert_near(params.lens[2], 0.75);
    assert_near(params.lens[3], 300.0);
    assert_near(params.coc_output[0], 6.0);
    assert_near(params.coc_output[1], 1.0 / 6.0);
    assert_near(params.coc_output[3], 1.0);
}

#[test]
fn depth_of_field_prepare_requires_aperture_and_radius() {
    assert!(!depth_of_field_prepare_enabled(
        RenderDepthOfFieldSettings::default()
    ));
    assert!(!depth_of_field_prepare_enabled(
        RenderDepthOfFieldSettings {
            aperture: 0.5,
            max_blur_radius: 0.0,
            ..Default::default()
        }
    ));
    assert!(!depth_of_field_prepare_enabled(
        RenderDepthOfFieldSettings {
            aperture: 0.0,
            max_blur_radius: 5.0,
            ..Default::default()
        }
    ));
    assert!(depth_of_field_prepare_enabled(RenderDepthOfFieldSettings {
        aperture: 0.5,
        max_blur_radius: 5.0,
        ..Default::default()
    }));
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.0001,
        "expected {actual} to be near {expected}"
    );
}
