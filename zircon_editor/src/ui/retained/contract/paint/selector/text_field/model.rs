//! 文本框的视觉结果覆盖表面、正文和步进器；绘制端负责当前内容、占位文字、裁剪与步进器几何。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

/// 供字段表面、正文和步进器绘制共享的视觉配方；内容和几何由调用端提供。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTextFieldStyle
{
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub stepper: [u8; 4],
    pub stepper_divider: [u8; 4],
    pub state: UiPainterResolvedState,
}
