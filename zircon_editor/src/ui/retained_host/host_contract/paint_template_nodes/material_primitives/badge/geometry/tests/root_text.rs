use super::*;

fn rect(width: f32) -> FrameRect {
    FrameRect {
        x: 2.0,
        y: 4.0,
        width,
        height: 28.0,
    }
}

fn node(font_size: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        font_size,
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn badge_root_text_frame_uses_runtime_text_width() {
    let node = node(12.0);
    let rect = rect(160.0);
    let label = "WWW iii";
    let text_frame = badge_root_text_frame(&node, &rect, label);
    let available_width = badge_root_available_text_width(&rect);
    let expected_width = badge_text_width(
        measure_runtime_text_width(label, text_frame.font_size),
        available_width,
    );

    assert!((text_frame.rect.width - expected_width).abs() <= 0.01);
    let old_heuristic_width = badge_text_width(
        label.chars().count() as f32 * text_frame.font_size * 0.56,
        available_width,
    );
    assert!((expected_width - old_heuristic_width).abs() > 0.25);
}

#[test]
fn badge_root_text_frame_clamps_measured_width_to_inset_width() {
    let node = node(12.0);
    let rect = rect(36.0);
    let text_frame = badge_root_text_frame(&node, &rect, "Long badge label");
    let available_width = badge_root_available_text_width(&rect);

    assert!((text_frame.rect.width - available_width).abs() <= 0.01);
}

#[test]
fn badge_root_text_frame_stays_inside_tight_root_bounds() {
    let node = node(0.0);
    let rect = FrameRect {
        x: 2.0,
        y: 4.0,
        width: 4.0,
        height: 6.0,
    };
    let text_frame = badge_root_text_frame(&node, &rect, "Badge");

    assert!(text_frame.rect.x >= rect.x);
    assert!(text_frame.rect.y >= rect.y);
    assert!(text_frame.rect.right() <= rect.right());
    assert!(text_frame.rect.bottom() <= rect.bottom());
}
