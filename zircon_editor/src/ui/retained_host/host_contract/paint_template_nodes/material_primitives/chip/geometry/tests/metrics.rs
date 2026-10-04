use super::*;

fn node(font_size: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        font_size,
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn chip_label_metrics_project_font_line_height_and_y() {
    let rect = FrameRect {
        x: 0.0,
        y: 10.0,
        width: 120.0,
        height: 32.0,
    };
    let line_height = chip_label_line_height(chip_font_size(&node(13.0), &rect), &rect);

    assert!((line_height - 19.5).abs() <= 0.01);
    assert!((chip_label_y(&rect, line_height) - 16.25).abs() <= 0.01);
}

#[test]
fn chip_label_width_clamps_to_available_bounds() {
    assert!((chip_label_width(80.0, 44.0) - 44.0).abs() <= 0.01);
    assert!((chip_label_width(0.0, 44.0) - 0.0).abs() <= 0.01);
}
