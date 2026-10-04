use super::{clamped_text_metrics, runtime_text_layout_frame};
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn retained_text_metrics_preserve_runtime_values_within_frame() {
    assert_eq!(clamped_text_metrics(18.0, 12.0, 14.0), (12.0, 14.0));
}

#[test]
fn retained_text_metrics_clamp_to_minimum_and_frame_height() {
    assert_eq!(clamped_text_metrics(0.0, 0.0, 0.0), (1.0, 1.0));
    assert_eq!(clamped_text_metrics(10.0, 12.0, 14.0), (10.0, 10.0));
}

#[test]
fn runtime_layout_frame_uses_minimum_positive_extent() {
    let frame = runtime_text_layout_frame(
        &FrameRect {
            x: 20.0,
            y: 30.0,
            width: 0.0,
            height: 8.0,
        },
        0.0,
    );

    assert_eq!(frame.x, 0.0);
    assert_eq!(frame.y, 0.0);
    assert_eq!(frame.width, 1.0);
    assert_eq!(frame.height, 1.0);
}
