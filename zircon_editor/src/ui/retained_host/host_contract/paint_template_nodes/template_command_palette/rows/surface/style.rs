//! 候选底面样式的只读快照；供对应 painter 在命令构造时消费。
//! 颜色与尺寸由上游主题或行状态传入，此层不持有交互状态或布局 owner。

use super::super::super::super::style_selector::WorkbenchPopupRowStyle;
use super::super::super::layout::WorkbenchCommandPaletteMetrics;

pub(super) struct CommandRowSurfaceStyle {
    pub fill: [u8; 4],
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn command_row_surface_style(
    style: WorkbenchPopupRowStyle,
    metrics: &WorkbenchCommandPaletteMetrics,
) -> Option<CommandRowSurfaceStyle> {
    let fill = style.background?;
    Some(CommandRowSurfaceStyle {
        fill,
        border: style.outline,
        border_width: if style.outline.is_some() {
            metrics.border_width
        } else {
            0.0
        },
        radius: metrics.row_radius,
    })
}
