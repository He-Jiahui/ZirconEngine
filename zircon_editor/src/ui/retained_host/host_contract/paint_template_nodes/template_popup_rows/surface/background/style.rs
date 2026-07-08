use super::super::super::super::super::paint_theme::{current_host_metrics, current_host_palette};

pub(super) struct PopupBackgroundStyle {
    pub fill: [u8; 4],
    pub border: [u8; 4],
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn popup_background_style() -> PopupBackgroundStyle {
    let palette = current_host_palette();
    let metrics = current_host_metrics();
    PopupBackgroundStyle {
        fill: palette.popup,
        border: palette.border,
        border_width: metrics.border_width,
        radius: metrics.radius_control,
    }
}
