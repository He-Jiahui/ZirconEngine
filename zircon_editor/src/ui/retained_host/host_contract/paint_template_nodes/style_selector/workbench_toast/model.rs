//! Toast 已解析视觉配方分开提供容器、状态标记、操作和关闭图标颜色；是否显示动作由绘制端和尺寸决定。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

/// Toast 容器及标记、操作、关闭通道的已解析视觉结果。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchToastStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub mark: [u8; 4],
    pub action: [u8; 4],
    pub close: [u8; 4],
    pub state: UiPainterResolvedState,
}
