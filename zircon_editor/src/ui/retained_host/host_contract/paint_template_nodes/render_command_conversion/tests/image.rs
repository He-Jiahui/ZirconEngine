use super::{raster_target_for_resource, FrameRect};

#[test]
fn runtime_resource_physical_size_precedes_the_logical_frame() {
    let frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 24.0,
        height: 16.0,
    };

    assert_eq!(
        raster_target_for_resource(Some((36.0, 24.0)), &frame),
        Some((36, 24))
    );
    assert_eq!(raster_target_for_resource(None, &frame), Some((24, 16)));
}
