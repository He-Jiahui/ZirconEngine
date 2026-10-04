use super::*;

fn rect() -> FrameRect {
    FrameRect {
        x: 20.0,
        y: 30.0,
        width: 80.0,
        height: 40.0,
    }
}

#[test]
fn badge_overlay_frame_uses_runtime_text_width_with_padding() {
    let node = TemplatePaneNodeData::default();
    let display = "WWW iii";
    let frame = badge_overlay_frame(&node, &rect(), display, false);
    let (expected_width, _) = badge_overlay_size(
        measure_runtime_text_width(display, badge_overlay_font_size()),
        false,
    );

    assert!((frame.width - expected_width).abs() <= 0.01);
    let (old_heuristic_width, _) = badge_overlay_size(
        display.chars().count() as f32 * badge_overlay_font_size() * 0.56,
        false,
    );
    assert!((expected_width - old_heuristic_width).abs() > 0.25);
}

#[test]
fn badge_overlay_text_frame_uses_runtime_text_width() {
    let display = "WWW iii";
    let overlay = FrameRect {
        width: 160.0,
        ..rect()
    };
    let text_frame = badge_overlay_text_frame(display, &overlay);
    let expected_width = badge_text_width(
        measure_runtime_text_width(display, badge_overlay_font_size()),
        overlay.width,
    );

    assert!((text_frame.rect.width - expected_width).abs() <= 0.01);
}

#[test]
fn badge_overlay_text_frame_clamps_measured_width_to_overlay_width() {
    let overlay = FrameRect {
        width: 20.0,
        ..rect()
    };
    let text_frame = badge_overlay_text_frame("Long badge overlay", &overlay);

    assert!((text_frame.rect.width - overlay.width).abs() <= 0.01);
}
