//! 图标按钮的工具栏、边栏、面板上下文决定是否绘制背景与边框；配方将这些可选通道交给模板画家。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchIconButtonContext
{
    Toolbar,
    Rail,
    Panel,
}

/// 图标按钮视觉配方；背景和边框缺失表示绘制端不输出对应通道。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchIconButtonStyle
{
    pub background: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub radius: f32,
    pub glyph: [u8; 4],
    pub state: UiPainterResolvedState,
}
