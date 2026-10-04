//! 复选框、单选框与开关的共同视觉结果；调用端提供具体种类，几何端按该种类消费标记或拇指通道。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

/// 调用端提供的选择控件类别，用于选择共享状态族与标记配方。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchSelectionControlKind
{
    Checkbox,
    Radio,
    Toggle,
}

/// 已解析的选择控件视觉通道；几何端按类别消费表面、标记或开关拇指。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSelectionControlStyle
{
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub thumb: [u8; 4],
    pub accent: [u8; 4],
    pub text: [u8; 4],
    pub label: [u8; 4],
    pub state: UiPainterResolvedState,
}
