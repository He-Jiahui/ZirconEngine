//! 查询与占位文字样式的只读快照；供对应 painter 在命令构造时消费。
//! 颜色与尺寸由上游主题或行状态传入，此层不持有交互状态或布局 owner。

use super::super::super::super::layout::WorkbenchCommandPaletteMetrics;
use super::super::super::super::palette::WorkbenchCommandPalettePalette;
use super::super::super::super::text::command_palette_text_style;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) struct CommandPaletteSearchTextStyle {
    pub color: [u8; 4],
    pub font_size: f32,
    pub line_height: f32,
    pub paint_style: UiTextRunPaintStyle,
}

pub(super) fn command_palette_search_text_style(
    palette: &WorkbenchCommandPalettePalette,
    metrics: &WorkbenchCommandPaletteMetrics,
    placeholder: bool,
) -> CommandPaletteSearchTextStyle {
    CommandPaletteSearchTextStyle {
        color: search_text_color(palette, placeholder),
        font_size: metrics.font_size,
        line_height: metrics.line_height,
        paint_style: command_palette_text_style(),
    }
}

fn search_text_color(palette: &WorkbenchCommandPalettePalette, placeholder: bool) -> [u8; 4] {
    if placeholder {
        palette.placeholder
    } else {
        palette.text
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
