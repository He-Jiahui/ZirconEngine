//! 弹层容器的宿主主题快照；投影了有效圆角时优先使用节点声明，否则退回panel级别半径。

use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::{current_host_metrics, current_host_palette};

pub(super) struct PopupBackgroundStyle {
    pub fill: [u8; 4],
    pub border: [u8; 4],
    pub border_width: f32,
    pub radius: f32,
}

pub(super) fn popup_background_style(node: &TemplatePaneNodeData) -> PopupBackgroundStyle {
    let palette = current_host_palette();
    let metrics = current_host_metrics();
    PopupBackgroundStyle {
        fill: palette.popup,
        border: palette.border,
        border_width: metrics.border_width,
        radius: if node.corner_radius.is_finite() && node.corner_radius > 0.0 {
            node.corner_radius
        } else {
            metrics.radius_panel
        },
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
