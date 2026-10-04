//! 表格行视觉结果携带行身份和文字角色；绘制端按列序调用 text_for_cell，必须与四列表格布局一致。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

/// 供表格外壳和四列文字绘制端共享的视觉配方；列索引契约见 text_for_cell。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTableRowStyle
{
    pub background: [u8; 4],
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub separator: [u8; 4],
    pub action: [u8; 4],
    pub state: UiPainterResolvedState,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) text: [u8; 4],
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) muted_text: [u8; 4],
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) tail_value_text: [u8; 4],
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) header: bool,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) tail: bool,
}
