use super::{union_optional_frames, visible_frame};
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn visible_frame_rejects_zero_and_non_finite_frames() {
    assert!(visible_frame(&rect(0.0, 0.0, 1.0, 1.0)));
    assert!(!visible_frame(&rect(0.0, 0.0, 0.0, 1.0)));
    assert!(!visible_frame(&rect(f32::NAN, 0.0, 1.0, 1.0)));
}

#[test]
fn union_optional_frames_preserves_single_frame_and_unions_pairs() {
    let left = rect(4.0, 8.0, 12.0, 10.0);
    let right = rect(10.0, 2.0, 30.0, 14.0);

    assert_eq!(union_optional_frames(Some(left.clone()), None), Some(left));
    assert_eq!(
        union_optional_frames(Some(rect(4.0, 8.0, 12.0, 10.0)), Some(right)),
        Some(rect(4.0, 2.0, 36.0, 16.0))
    );
    assert_eq!(union_optional_frames(None, None), None);
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
