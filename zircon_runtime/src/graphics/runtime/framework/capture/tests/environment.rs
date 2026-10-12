use super::{renderdoc_capture_frame_count_from_values, MAX_RENDERDOC_CAPTURE_FRAME_COUNT};

#[test]
fn renderdoc_capture_frame_count_preserves_the_legacy_next_frame_switch() {
    assert_eq!(
        renderdoc_capture_frame_count_from_values(None, Some("1")),
        1
    );
    assert_eq!(renderdoc_capture_frame_count_from_values(None, None), 0);
}

#[test]
fn renderdoc_capture_frame_count_is_explicit_and_bounded() {
    assert_eq!(
        renderdoc_capture_frame_count_from_values(Some("2"), Some("1")),
        2
    );
    assert_eq!(
        renderdoc_capture_frame_count_from_values(Some("0"), Some("1")),
        0
    );
    assert_eq!(
        renderdoc_capture_frame_count_from_values(Some("999"), None),
        MAX_RENDERDOC_CAPTURE_FRAME_COUNT
    );
    assert_eq!(
        renderdoc_capture_frame_count_from_values(Some("invalid"), Some("1")),
        1
    );
}
