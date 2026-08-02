use super::super::super::super::paint_theme::{HostControlMetrics, current_host_metrics};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct DragOverlayMetrics {
    pub border_width: f32,
    pub preview_radius: f32,
    pub icon_radius: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub icon_left: f32,
    pub icon_size: f32,
    pub text_left_with_icon: f32,
    pub text_right_inset: f32,
    pub indicator_thickness: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn drag_overlay_metrics()
-> DragOverlayMetrics {
    drag_overlay_metrics_from_host(current_host_metrics())
}

fn drag_overlay_metrics_from_host(metrics: HostControlMetrics) -> DragOverlayMetrics {
    let icon_size = (metrics.row_height - metrics.gap_l).max(0.0);
    DragOverlayMetrics {
        border_width: metrics.border_width,
        preview_radius: metrics.radius_control,
        icon_radius: metrics.radius_control.min(icon_size * 0.5),
        font_size: metrics.font_body,
        line_height: metrics.line_height(metrics.font_body),
        icon_left: metrics.button_pad_x,
        icon_size,
        text_left_with_icon: metrics.button_pad_x + icon_size + metrics.button_icon_gap,
        text_right_inset: metrics.button_pad_x,
        indicator_thickness: metrics.tab_underline_height,
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::paint_theme::METRICS;
    use super::*;

    #[test]
    fn drag_overlay_metrics_project_from_shared_host_control_tokens() {
        let mut host = METRICS;
        host.border_width = 1.5;
        host.radius_control = 5.0;
        host.font_body = 12.0;
        host.line_height_ratio = 1.25;
        host.button_pad_x = 14.0;
        host.button_icon_gap = 6.0;
        host.gap_l = 13.0;
        host.row_height = 30.0;
        host.tab_underline_height = 3.0;

        let overlay = drag_overlay_metrics_from_host(host);

        assert_eq!(overlay.border_width, 1.5);
        assert_eq!(overlay.preview_radius, 5.0);
        assert_eq!(overlay.icon_radius, 5.0);
        assert_eq!(overlay.font_size, 12.0);
        assert_eq!(overlay.line_height, 15.0);
        assert_eq!(overlay.icon_left, 14.0);
        assert_eq!(overlay.icon_size, 17.0);
        assert_eq!(overlay.text_left_with_icon, 37.0);
        assert_eq!(overlay.text_right_inset, 14.0);
        assert_eq!(overlay.indicator_thickness, 3.0);
    }
}
