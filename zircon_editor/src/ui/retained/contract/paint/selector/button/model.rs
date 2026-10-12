//! 按钮类型与绘制配方供模板按钮画家共享；交互状态随配方传递，颜色与边宽由选择器统一确定。

use zircon_runtime_interface::ui::style::ButtonInteractionState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchButtonKind {
    Primary,
    Secondary,
    Tertiary,
    Danger,
}

/// 模板按钮共享的已解析绘制配方；交互态供绘制端使用，颜色和边宽由选择器确定。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchButtonStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub border_width: f32,
    pub text: [u8; 4],
    pub glyph: [u8; 4],
    pub interaction: ButtonInteractionState,
}
