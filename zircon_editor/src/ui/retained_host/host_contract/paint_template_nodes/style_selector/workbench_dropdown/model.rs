//! 下拉框绘制配方同时承载表面、边框、文字、箭头和已解析状态，供模板绘制端一致消费。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

/// 下拉框各视觉通道的统一配方，供表面、文字与箭头绘制端消费。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchDropdownStyle
{
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub chevron: [u8; 4],
    pub state: UiPainterResolvedState,
}
