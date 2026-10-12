//! 搜索焦点底面样式的只读快照；供对应 painter 在命令构造时消费。
//! 颜色与尺寸由上游主题或行状态传入，此层不持有交互状态或布局 owner。

use super::super::super::super::layout::WorkbenchCommandPaletteMetrics;
use super::super::super::super::palette::WorkbenchCommandPalettePalette;

pub(super) struct CommandPaletteSearchSurfaceStyle {
    pub fill: [u8; 4],
    pub border: [u8; 4],
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn command_palette_search_surface_style(
    palette: &WorkbenchCommandPalettePalette,
    metrics: &WorkbenchCommandPaletteMetrics,
    focused: bool,
) -> CommandPaletteSearchSurfaceStyle {
    CommandPaletteSearchSurfaceStyle {
        fill: palette.search_surface,
        border: search_border_color(palette, focused),
        border_width: metrics.border_width,
        radius: metrics.search_radius,
    }
}

fn search_border_color(palette: &WorkbenchCommandPalettePalette, focused: bool) -> [u8; 4] {
    if focused {
        palette.search_focus_border
    } else {
        palette.search_idle_border
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
