use super::*;

fn rect(width: f32) -> FrameRect {
    FrameRect {
        x: 4.0,
        y: 6.0,
        width,
        height: 48.0,
    }
}

fn node(font_size: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        font_size,
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn avatar_text_frame_uses_runtime_text_width() {
    let node = node(18.0);
    let rect = rect(180.0);
    let label = "WWW iii";
    let (frame, font_size, _) = avatar_text_frame(&node, &rect, label);
    let expected_width =
        avatar_text_width(measure_runtime_text_width(label, font_size), rect.width);

    assert!((frame.width - expected_width).abs() <= 0.01);
    assert!((frame.x - avatar_centered_text_x(&rect, expected_width)).abs() <= 0.01);
    let old_heuristic_width =
        avatar_text_width(label.chars().count() as f32 * font_size * 0.58, rect.width);
    assert!((expected_width - old_heuristic_width).abs() > 0.25);
}

#[test]
fn avatar_text_frame_clamps_measured_width_to_avatar_width() {
    let node = node(18.0);
    let rect = rect(32.0);
    let (frame, _, _) = avatar_text_frame(&node, &rect, "Long avatar label");

    assert!((frame.width - rect.width).abs() <= 0.01);
    assert!((frame.x - rect.x).abs() <= 0.01);
}
