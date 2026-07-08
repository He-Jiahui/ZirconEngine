use super::super::super::layout::WorkbenchCommandPaletteMetrics;
use super::super::super::text::command_palette_text_style;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) struct CommandRowDetailTextStyle {
    pub color: [u8; 4],
    pub font_size: f32,
    pub line_height: f32,
    pub paint_style: UiTextRunPaintStyle,
}

pub(super) fn command_row_detail_text_style(
    color: [u8; 4],
    metrics: &WorkbenchCommandPaletteMetrics,
) -> CommandRowDetailTextStyle {
    CommandRowDetailTextStyle {
        color,
        font_size: metrics.font_size,
        line_height: metrics.line_height,
        paint_style: command_palette_text_style(),
    }
}
