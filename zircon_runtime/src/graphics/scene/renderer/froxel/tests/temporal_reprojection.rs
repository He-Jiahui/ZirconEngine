use super::*;
use crate::core::framework::render::{FroxelGridQuality, TemporalJitterSample};
use crate::core::math::Vec2;

#[test]
fn temporal_reprojection_consumes_camera_jitter_only_when_history_is_available() {
    let current = ViewportCameraSnapshot {
        temporal_jitter: TemporalJitterSample {
            offset_pixels: Vec2::new(0.25, -0.125),
            sequence_index: 3,
        },
        ..ViewportCameraSnapshot::default()
    };
    let grid = FroxelGridParams::for_quality(FroxelGridQuality::High, 0.1, 1000.0, 2.0);

    let unavailable = GpuFroxelTemporalReprojection::new(
        &current,
        None,
        UVec2::new(1600, 900),
        grid,
        true,
        false,
    );
    let available = GpuFroxelTemporalReprojection::new(
        &current,
        Some(&current),
        UVec2::new(1600, 900),
        grid,
        true,
        true,
    );

    assert_eq!(unavailable.jitter_and_history[0], 0.25);
    assert_eq!(unavailable.jitter_and_history[1], -0.125);
    assert_eq!(unavailable.jitter_and_history[3], 0.0);
    assert_eq!(available.jitter_and_history[3], 0.9);
    assert!((available.jitter_and_history[2] - 0.1).abs() <= 0.000001);
}
