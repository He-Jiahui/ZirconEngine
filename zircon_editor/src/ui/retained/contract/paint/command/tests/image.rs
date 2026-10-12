use super::*;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

#[test]
fn render_command_image_border_projects_from_host_palette() {
    let mut palette = PALETTE;
    palette.border = [10, 11, 12, 255];
    palette.focus_ring = [13, 14, 15, 255];

    assert_eq!(fallback_image_border_from_host(palette), [10, 11, 12, 255]);
}

#[test]
fn render_command_image_frame_metrics_project_from_host_metrics() {
    assert_eq!(
        fallback_image_frame_metrics_from_host(METRICS),
        (METRICS.border_width, METRICS.radius_control)
    );
}

#[test]
fn render_command_image_fallback_text_metrics_project_from_host_metrics() {
    assert_eq!(
        HostPaintCommand::fallback_text_metrics_from_host(METRICS),
        (METRICS.font_body, METRICS.line_height(METRICS.font_body))
    );
}
