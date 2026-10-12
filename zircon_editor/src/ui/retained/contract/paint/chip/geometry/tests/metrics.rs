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

#[test]
fn authored_chip_font_scales_once_and_physical_fallback_is_preserved() {
    use crate::ui::retained_host::host_contract::paint_theme::{
        current_host_metrics, enter_host_paint_theme_scope,
        host_paint_theme_snapshot_at_scale_for_test,
    };
    for scale in [1.0, 1.5] {
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));

        let rect = FrameRect {
            width: 120.0,
            height: 40.0,
            ..Default::default()
        };
        assert_eq!(chip_font_size(&node(12.0), &rect), 12.0 * scale);
        // The existing component fallback is a physical metric.
        assert_eq!(chip_font_size(&node(0.0), &rect), CHIP_LABEL_FONT_SIZE);
        assert_eq!(
            chip_font_size(&node(f32::INFINITY), &rect),
            CHIP_LABEL_FONT_SIZE
        );
        assert_eq!(current_host_metrics().scale_factor, scale);
    }
}
