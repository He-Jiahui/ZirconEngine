use super::*;

fn rect(width: f32) -> FrameRect {
    FrameRect {
        x: 8.0,
        y: 10.0,
        width,
        height: 32.0,
    }
}

fn node(font_size: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        font_size,
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn chip_label_frame_uses_runtime_text_width() {
    let node = node(13.0);
    let label = "WWW iii";
    let frame = chip_label_frame(&node, &rect(220.0), label)
        .expect("chip label has enough horizontal space")
        .0;
    let expected_width = chip_label_width(
        measure_runtime_text_width(label, chip_font_size(&node, &rect(220.0))),
        220.0 - chip_label_left_padding(&node) - chip_label_right_padding(&node),
    );

    assert!(
        (frame.width - expected_width).abs() <= 0.01,
        "chip label frame must use runtime text measurement width"
    );
    let old_heuristic_width = chip_label_width(label.chars().count() as f32 * 13.0 * 0.56, 220.0);
    assert!(
        (expected_width - old_heuristic_width).abs() > 0.25,
        "fixture should catch regressions back to char-count width"
    );
}

#[test]
fn chip_label_frame_clamps_measured_width_to_available_space() {
    let node = node(13.0);
    let available_width = 44.0 - chip_label_left_padding(&node) - chip_label_right_padding(&node);
    let frame = chip_label_frame(&node, &rect(44.0), "Long chip label")
        .expect("chip label has enough horizontal space")
        .0;

    assert!((frame.width - available_width).abs() <= 0.01);
}

#[test]
fn chip_label_frame_stays_inside_short_chip_bounds() {
    let node = node(0.0);
    let rect = FrameRect {
        x: 8.0,
        y: 10.0,
        width: 40.0,
        height: 0.6,
    };
    let frame = chip_label_frame(&node, &rect, "Chip")
        .expect("chip has enough horizontal label space")
        .0;

    assert!(frame.x >= rect.x);
    assert!(frame.y >= rect.y);
    assert!(frame.right() <= rect.right());
    assert!(frame.bottom() <= rect.bottom());
}
