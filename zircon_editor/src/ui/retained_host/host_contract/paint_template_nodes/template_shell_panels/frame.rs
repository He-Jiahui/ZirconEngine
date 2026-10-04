//! 工作台 chrome 默认只画共享分隔线；内容面板额外拥有闭合轮廓和圆角，避免外壳容器边框重复。

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
#[path = "tests/frame.rs"]
mod tests;
