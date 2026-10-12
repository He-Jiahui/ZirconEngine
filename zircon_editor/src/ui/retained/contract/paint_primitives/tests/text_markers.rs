use super::{label_marker_frame, text_bar_frame_width, text_bar_line_height, text_bars_frame};
use crate::ui::retained_host::host_contract::{data::FrameRect, paint_theme::METRICS};

#[test]
fn text_bars_frame_uses_runtime_text_measurement() {
    let narrow = text_bars_frame(0.0, 0.0, "iiiiiiiiiiii", METRICS);
    let wide = text_bars_frame(0.0, 0.0, "WWWWWWWWWWWW", METRICS);

    assert!(
        wide.width > narrow.width + 8.0,
        "same-character-count text bars should follow runtime glyph width, narrow={narrow:?}, wide={wide:?}"
    );
    assert_eq!(wide.height, text_bar_line_height(METRICS));
}

#[test]
fn text_bars_frame_reserves_trailing_glyph_clip_guard() {
    assert_eq!(text_bar_frame_width(80.0, 6.0), 86.0);
    assert_eq!(text_bar_frame_width(0.0, -4.0), 1.0);
}

#[test]
fn label_marker_frame_uses_host_metrics_without_overflowing_the_target() {
    let target = FrameRect {
        x: 4.0,
        y: 8.0,
        width: 60.0,
        height: 10.0,
    };
    let frame = label_marker_frame(&target, METRICS);

    assert_eq!(frame.x, target.x + METRICS.button_pad_x);
    assert!(frame.y >= target.y);
    assert!(frame.x + frame.width <= target.x + target.width);
    assert!(frame.y + frame.height <= target.y + target.height);

    let narrow = FrameRect {
        width: 12.0,
        ..target
    };
    let narrow_frame = label_marker_frame(&narrow, METRICS);
    assert!(narrow_frame.x + narrow_frame.width <= narrow.x + narrow.width);
}
