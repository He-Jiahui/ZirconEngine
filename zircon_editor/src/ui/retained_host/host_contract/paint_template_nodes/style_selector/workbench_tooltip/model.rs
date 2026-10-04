//! Tooltip 视觉配方分开携带气泡、箭头、图标、正文与阴影；位置与显示条件交给模板绘制端。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

/// 供提示气泡、箭头、图标、正文和阴影绘制端共享的视觉结果。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTooltipStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub title: [u8; 4],
    pub body: [u8; 4],
    pub arrow: [u8; 4],
    pub icon: [u8; 4],
    pub shadow: [u8; 4],
    pub state: UiPainterResolvedState,
}
