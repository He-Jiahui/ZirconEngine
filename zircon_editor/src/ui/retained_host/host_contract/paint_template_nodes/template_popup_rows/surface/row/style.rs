use super::super::super::super::style_selector::WorkbenchPopupRowStyle;
use super::super::super::metrics::WorkbenchPopupRowMetrics;

pub(super) struct PopupRowSurfaceCommandStyle {
    pub fill: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn popup_row_surface_command_style(
    style: WorkbenchPopupRowStyle,
    metrics: &WorkbenchPopupRowMetrics,
) -> Option<PopupRowSurfaceCommandStyle> {
    if style.background.is_none() && style.outline.is_none() {
        return None;
    }
    Some(PopupRowSurfaceCommandStyle {
        fill: style.background,
        border: style.outline,
        border_width: popup_row_surface_border_width(&style, metrics),
        radius: metrics.surface_radius,
    })
}

fn popup_row_surface_border_width(
    style: &WorkbenchPopupRowStyle,
    metrics: &WorkbenchPopupRowMetrics,
) -> f32 {
    if style.outline.is_some() {
        metrics.outline_width
    } else {
        0.0
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
