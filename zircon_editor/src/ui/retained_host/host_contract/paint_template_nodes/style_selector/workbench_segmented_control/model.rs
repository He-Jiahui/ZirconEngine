//! 模板分段与页签共享的视觉配方；普通外壳、选中标记和组标签保持独立，供对应绘制端按选中项使用。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

/// 调用端识别的绘制类别；分段容器与单个页签共享颜色配方但使用不同外壳通道。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchSegmentedControlKind
{
    SegmentedControl,
    Tab,
}

/// 供分段与页签绘制端按选中项消费的视觉配方；选中通道不会自动决定选择。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSegmentedControlStyle
{
    pub background: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub selected_surface: [u8; 4],
    pub selected_border: [u8; 4],
    pub selected_border_width: f32,
    pub selected_underline: [u8; 4],
    pub selected_underline_height: f32,
    pub selected_text: [u8; 4],
    pub idle_text: [u8; 4],
    pub group_label: [u8; 4],
    pub state: UiPainterResolvedState,
}
