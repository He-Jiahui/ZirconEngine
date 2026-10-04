//! 警报语义色调与绘制配方分离；模板警报画家消费配方，不由调用端重复选择状态色。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchAlertTone {
    Info,
    Success,
    Warning,
    Error,
}

/// 模板警报的已解析视觉配方；语义色调与运行时状态已在选择器中合并。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchAlertStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub mark: [u8; 4],
    pub text: [u8; 4],
    pub state: UiPainterResolvedState,
}
