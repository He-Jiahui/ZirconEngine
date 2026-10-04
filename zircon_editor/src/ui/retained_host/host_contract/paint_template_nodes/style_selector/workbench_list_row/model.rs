//! 列表行的已解析视觉配方；无表面或边线由绘制端跳过对应四边形通道。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

/// 供列表行绘制端消费的视觉结果；整行与装饰标记的选择语义独立。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchListRowStyle {
    pub background: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub text: [u8; 4],
    pub adornment: [u8; 4],
    pub state: UiPainterResolvedState,
}
