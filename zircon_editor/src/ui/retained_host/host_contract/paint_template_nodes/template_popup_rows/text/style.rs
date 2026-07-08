use super::super::metrics::WorkbenchPopupRowMetrics;
use super::popup_row_text_style;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) struct PopupRowTextCommandStyle {
    pub color: [u8; 4],
    pub font_size: f32,
    pub line_height: f32,
    pub paint_style: UiTextRunPaintStyle,
}

pub(super) fn popup_row_text_command_style(
    color: [u8; 4],
    metrics: &WorkbenchPopupRowMetrics,
) -> PopupRowTextCommandStyle {
    PopupRowTextCommandStyle {
        color,
        font_size: metrics.font_size,
        line_height: metrics.line_height,
        paint_style: popup_row_text_style(),
    }
}
