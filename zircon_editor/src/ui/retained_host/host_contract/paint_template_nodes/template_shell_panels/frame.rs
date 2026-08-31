use super::super::super::paint_theme::{current_host_metrics, HostControlMetrics};
use super::super::style_selector::{WorkbenchChromeKind as ShellPanelKind, WorkbenchChromeStyle};

#[derive(Clone, Copy, Debug, PartialEq)]
struct ShellPanelFrameMetrics {
    border_width: f32,
    corner_radius: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn shell_panel_border_color(
    kind: ShellPanelKind,
    style: &WorkbenchChromeStyle,
) -> Option<[u8; 4]> {
    shell_panel_draws_frame(kind).then_some(style.separator)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn shell_panel_border_width(
    kind: ShellPanelKind,
) -> f32 {
    if shell_panel_draws_frame(kind) {
        shell_panel_frame_metrics().border_width
    } else {
        0.0
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn shell_panel_corner_radius(
    kind: ShellPanelKind,
) -> f32 {
    if shell_panel_draws_frame(kind) {
        shell_panel_frame_metrics().corner_radius
    } else {
        0.0
    }
}

fn shell_panel_frame_metrics() -> ShellPanelFrameMetrics {
    shell_panel_frame_metrics_from_host(current_host_metrics())
}

fn shell_panel_frame_metrics_from_host(metrics: HostControlMetrics) -> ShellPanelFrameMetrics {
    ShellPanelFrameMetrics {
        border_width: metrics.border_width,
        corner_radius: metrics.radius_control,
    }
}

fn shell_panel_draws_frame(kind: ShellPanelKind) -> bool {
    matches!(kind, ShellPanelKind::ContentPanel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_panel_frame_metrics_project_from_host_control_metrics() {
        let metrics = HostControlMetrics {
            control_default_height: 32.0,
            control_large_height: 48.0,
            radius_small: 2.0,
            radius_control: 3.0,
            radius_panel: 4.0,
            border_width: 1.5,
            font_small: 8.0,
            font_body: 10.0,
            font_large: 14.0,
            line_height_ratio: 1.2,
            button_pad_x: 12.0,
            button_icon_gap: 7.0,
            button_chevron_reserve: 18.0,
            text_clip_guard: 6.0,
            button_pressed_offset_y: 1.0,
            input_pad: [8.0, 8.0, 3.0, 4.0],
            segment_text_inset_y: 4.0,
            segment_selected_inset: 2.0,
            tab_underline_height: 2.0,
            selection_indicator_width: 2.0,
            scrollbar_thickness: 8.0,
            scrollbar_min_thumb_length: 24.0,
            gap_s: 4.0,
            gap_m: 8.0,
            gap_l: 12.0,
            row_height: 24.0,
        };

        let frame = shell_panel_frame_metrics_from_host(metrics);

        assert_eq!(frame.border_width, 1.5);
        assert_eq!(frame.corner_radius, 3.0);
    }
}
