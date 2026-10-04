use super::*;
use std::num::{NonZeroU16, NonZeroU32};
use winit::dpi::PhysicalSize;

#[test]
fn video_mode_matching_treats_unspecified_fields_as_wildcards() {
    let candidate = VideoMode::new(
        PhysicalSize::new(1920, 1080),
        NonZeroU16::new(32),
        NonZeroU32::new(60_000),
    );

    assert!(video_mode_matches(
        &candidate,
        WindowVideoMode::new(1920, 1080)
    ));
    assert!(video_mode_matches(
        &candidate,
        WindowVideoMode::new(1920, 1080)
            .with_bit_depth(32)
            .with_refresh_rate_millihertz(60_000)
    ));
    assert!(!video_mode_matches(
        &candidate,
        WindowVideoMode::new(1280, 720)
    ));
    assert!(!video_mode_matches(
        &candidate,
        WindowVideoMode::new(1920, 1080).with_bit_depth(24)
    ));
    assert!(!video_mode_matches(
        &candidate,
        WindowVideoMode::new(1920, 1080).with_refresh_rate_millihertz(59_940)
    ));
}
