//! 容器底面样式的只读快照；供对应 painter 在命令构造时消费。
//! 颜色与尺寸由上游主题或行状态传入，此层不持有交互状态或布局 owner。

use super::super::super::layout::WorkbenchCommandPaletteMetrics;
use super::super::super::palette::WorkbenchCommandPalettePalette;

pub(super) struct CommandPalettePanelSurfaceStyle {
    pub fill: [u8; 4],
    pub border: [u8; 4],
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn command_palette_panel_surface_style(
    palette: &WorkbenchCommandPalettePalette,
    metrics: &WorkbenchCommandPaletteMetrics,
) -> CommandPalettePanelSurfaceStyle {
    CommandPalettePanelSurfaceStyle {
        fill: palette.panel_surface,
        border: palette.panel_border,
        border_width: metrics.border_width,
        radius: metrics.panel_radius,
    }
}
