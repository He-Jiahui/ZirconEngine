//! 固定状态栏控制 ID 对应交互类型；诊断项则从验证等级导出语义信号，和状态选择器保持一致。

use super::super::super::data::TemplatePaneNodeData;
use super::super::style_selector::{
    WorkbenchStatusSignalKind as StatusSignalKind, WORKBENCH_DIAGNOSTIC_SIGNAL_VARIANT,
};
use super::super::template_status_glyphs::StatusIconKind;

/// 专用状态控件的绘制所有权标记；认领后不会再落入模板节点通用表面分支。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum StatusControlKind {
    Signal(StatusSignalKind),
    Chip,
    Icon(StatusIconKind),
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_control_kind(
    node: &TemplatePaneNodeData,
) -> Option<StatusControlKind> {
    match node.control_id.as_str() {
        "WorkbenchStatusReady" => Some(StatusControlKind::Signal(StatusSignalKind::Ready)),
        "WorkbenchStatusErrors" => Some(StatusControlKind::Signal(StatusSignalKind::Success)),
        "WorkbenchStatusWarnings" => Some(StatusControlKind::Signal(StatusSignalKind::Warning)),
        "WorkbenchStatusMessages" => Some(StatusControlKind::Signal(StatusSignalKind::Info)),
        "WorkbenchStatusGrid" | "WorkbenchStatusSnap" | "WorkbenchStatusZoom" => {
            Some(StatusControlKind::Chip)
        }
        "WorkbenchStatusSnapToggle" => Some(StatusControlKind::Icon(StatusIconKind::Snap)),
        "WorkbenchStatusWorld" => Some(StatusControlKind::Icon(StatusIconKind::World)),
        "WorkbenchStatusTarget" => Some(StatusControlKind::Icon(StatusIconKind::Target)),
        _ if node.component_variant.as_str() == WORKBENCH_DIAGNOSTIC_SIGNAL_VARIANT => {
            Some(StatusControlKind::Signal(diagnostic_signal_kind(node)))
        }
        _ => None,
    }
}

fn diagnostic_signal_kind(node: &TemplatePaneNodeData) -> StatusSignalKind {
    match node.validation_level.as_str() {
        "success" => StatusSignalKind::Success,
        "warning" => StatusSignalKind::Warning,
        "error" | "danger" => StatusSignalKind::Error,
        _ => StatusSignalKind::Info,
    }
}
