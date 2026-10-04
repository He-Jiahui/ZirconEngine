use crate::core::framework::render::{
    TemporalJitterSample, ViewProjectionMatrixPair, ViewportCameraSnapshot,
};
use crate::core::math::{UVec2, Vec2, Vec3};

#[test]
fn render_taa_matrix_pair_is_identical_without_jitter() {
    let source = include_str!("../view_matrix_pair.rs");
    assert!(source.contains(concat!("if jitter == Vec2::", "ZERO {")));

    let camera = ViewportCameraSnapshot::default();

    let pair = ViewProjectionMatrixPair::from_camera(&camera, UVec2::new(1280, 720));

    assert_eq!(
        pair.clip_from_world_jittered,
        pair.clip_from_world_unjittered
    );
}

#[test]
fn render_taa_matrix_pair_applies_pixel_jitter_in_clip_space() {
    let camera = ViewportCameraSnapshot {
        temporal_jitter: TemporalJitterSample {
            offset_pixels: Vec2::new(0.5, -0.25),
            sequence_index: 3,
        },
        ..ViewportCameraSnapshot::default()
    };

    let pair = ViewProjectionMatrixPair::from_camera(&camera, UVec2::new(100, 50));
    let world_point = Vec3::new(0.0, 0.0, -1.0);
    let unjittered_origin = pair.clip_from_world_unjittered.project_point3(world_point);
    let jittered_origin = pair.clip_from_world_jittered.project_point3(world_point);

    assert_close(jittered_origin.x - unjittered_origin.x, 0.01);
    assert_close(jittered_origin.y - unjittered_origin.y, -0.01);
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.0001,
        "expected {actual} to be close to {expected}"
    );
}
