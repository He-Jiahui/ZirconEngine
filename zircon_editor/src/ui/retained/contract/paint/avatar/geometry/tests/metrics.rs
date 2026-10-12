use super::*;

fn node(font_size: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        font_size,
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn avatar_text_metrics_project_font_width_and_centering() {
    let rect = FrameRect {
        x: 4.0,
        y: 6.0,
        width: 48.0,
        height: 48.0,
    };
    let font_size = avatar_font_size(&node(0.0), &rect);
    let text_width = avatar_text_width(18.0, rect.width);

    assert!((font_size - 24.0).abs() <= 0.01);
    assert!((avatar_text_line_height(font_size) - 24.0).abs() <= 0.01);
    assert!((avatar_centered_text_x(&rect, text_width) - 19.0).abs() <= 0.01);
    assert!((avatar_centered_text_y(&rect, font_size) - 18.0).abs() <= 0.01);
}

#[test]
fn avatar_text_width_clamps_to_avatar_bounds() {
    assert!((avatar_text_width(80.0, 32.0) - 32.0).abs() <= 0.01);
    assert!((avatar_text_width(0.0, 32.0) - 0.0).abs() <= 0.01);
    assert!((avatar_text_width(12.0, 0.4) - 0.4).abs() <= 0.01);
}

#[test]
fn authored_avatar_font_scales_once_and_physical_fallback_is_preserved() {
    use crate::ui::retained_host::host_contract::paint_theme::{
        current_host_metrics, enter_host_paint_theme_scope,
        host_paint_theme_snapshot_at_scale_for_test,
    };
    for scale in [1.0, 1.5] {
        let _scope =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));

        let rect = FrameRect {
            width: 48.0,
            height: 48.0,
            ..Default::default()
        };
        assert_eq!(avatar_font_size(&node(12.0), &rect), 12.0 * scale);
        // Fallback derives from the physical frame, so must not be scaled again.
        assert_eq!(avatar_font_size(&node(0.0), &rect), 24.0);
        assert_eq!(avatar_font_size(&node(f32::NAN), &rect), 24.0);
        assert_eq!(current_host_metrics().scale_factor, scale);
    }
}
