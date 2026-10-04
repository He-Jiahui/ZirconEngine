use super::*;
use crate::ui::retained_host::host_contract::paint_theme::METRICS;

fn node(font_size: f32) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        font_size,
        ..TemplatePaneNodeData::default()
    }
}

#[test]
fn badge_root_text_metrics_project_font_line_height_and_width() {
    let rect = FrameRect {
        x: 2.0,
        y: 4.0,
        width: 160.0,
        height: 28.0,
    };
    let font_size = badge_root_font_size(&node(12.0), &rect);

    assert!((font_size - 12.0).abs() <= 0.01);
    assert!((badge_text_line_height(font_size, &rect) - 14.4).abs() <= 0.01);
    assert!((badge_root_available_text_width(&rect) - 144.0).abs() <= 0.01);
    assert!((badge_root_text_x(&rect) - 10.0).abs() <= 0.01);
}

#[test]
fn badge_root_font_size_tracks_host_typography_and_tight_bounds() {
    let mut host = METRICS;
    host.font_body = 11.0;
    let wide = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 80.0,
        height: 24.0,
    };
    let tight = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 4.0,
        height: 6.0,
    };
    let node = node(0.0);

    assert_eq!(badge_root_font_size_from_host(&node, &wide, host), 11.0);
    assert_eq!(badge_root_font_size_from_host(&node, &tight, host), 4.0);
}

#[test]
fn badge_overlay_metrics_project_text_size_and_rect() {
    let (width, height) = badge_overlay_size(24.0, false);
    let rect = badge_overlay_rect(40.0, 30.0, width, height);

    assert!((width - 36.0).abs() <= 0.01);
    assert!((height - 20.0).abs() <= 0.01);
    assert!((rect.x - 22.0).abs() <= 0.01);
    assert!((rect.y - 20.0).abs() <= 0.01);
}

#[test]
fn badge_overlay_preserves_fractional_post_dpi_geometry() {
    let rect = badge_overlay_rect(40.25, 30.75, 36.5, 20.5);

    assert_eq!(rect.x, 22.0);
    assert_eq!(rect.y, 20.5);
    assert_eq!(rect.width, 36.5);
    assert_eq!(rect.height, 20.5);
}

#[test]
fn badge_text_width_clamps_to_available_bounds() {
    assert!((badge_text_width(80.0, 20.0) - 20.0).abs() <= 0.01);
    assert!((badge_text_width(0.0, 20.0) - 0.0).abs() <= 0.01);
}
