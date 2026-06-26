use super::super::super::paint_theme::METRICS;
use super::super::style_selector::{WorkbenchChromeKind as ShellPanelKind, WorkbenchChromeStyle};

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
        METRICS.border_width
    } else {
        0.0
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn shell_panel_corner_radius(
    kind: ShellPanelKind,
) -> f32 {
    if shell_panel_draws_frame(kind) {
        METRICS.radius_control
    } else {
        0.0
    }
}

fn shell_panel_draws_frame(kind: ShellPanelKind) -> bool {
    matches!(kind, ShellPanelKind::ContentPanel)
}
