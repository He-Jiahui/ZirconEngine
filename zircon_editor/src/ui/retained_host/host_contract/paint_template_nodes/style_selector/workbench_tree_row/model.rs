//! 树行的已解析视觉配方；透明普通表面以 None 表达，四种内容通道供不同绘制节点使用。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

/// 供树行外壳与多种内容通道消费的视觉结果；无背景/边线表示绘制端跳过。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTreeRowStyle {
    pub background: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub text: [u8; 4],
    pub icon: [u8; 4],
    pub secondary: [u8; 4],
    pub action: [u8; 4],
    pub state: UiPainterResolvedState,
}
