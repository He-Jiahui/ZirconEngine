//! 候选辅助列样式的只读快照；供对应 painter 在命令构造时消费。
//! 颜色与尺寸由上游主题或行状态传入，此层不持有交互状态或布局 owner。

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
