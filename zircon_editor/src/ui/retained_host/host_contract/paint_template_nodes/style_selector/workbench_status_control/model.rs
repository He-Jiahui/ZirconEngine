//! 状态条视觉契约区分语义信号、诊断信号、文本胶囊与图标按钮；种类和变体由调用端的身份判定提供。

use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DIAGNOSTIC_SIGNAL_VARIANT:
    &str = "diagnostic_signal";
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_SEMANTIC_STATUS_SIGNAL_VARIANT:
    &str = "semantic_status_signal";

/// 由状态控件身份端选择的信号含义，供信号图标与文字配方共同使用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchStatusSignalKind
{
    Ready,
    Success,
    Warning,
    Info,
    Error,
}

/// 信号图标和文字的已解析视觉结果。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchStatusSignalStyle
{
    pub icon_fill: [u8; 4],
    pub text: [u8; 4],
    pub state: UiPainterResolvedState,
}

/// 状态文本胶囊的视觉结果，标签和数值文字通道分别消费。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchStatusChipStyle
{
    pub background: [u8; 4],
    pub border: [u8; 4],
    pub label_text: [u8; 4],
    pub value_text: [u8; 4],
    pub state: UiPainterResolvedState,
}

/// 状态条图标按钮的视觉结果，透明外壳由绘制端过滤。
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchStatusIconButtonStyle
{
    pub background: [u8; 4],
    pub border: [u8; 4],
    pub glyph: [u8; 4],
    pub state: UiPainterResolvedState,
}
