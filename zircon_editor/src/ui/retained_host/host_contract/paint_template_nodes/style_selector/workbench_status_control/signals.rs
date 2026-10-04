//! 状态信号选择入口接收调用端已判定的种类；变体区分诊断严重程度着色与语义状态图标、正文配方。
//! 指定信号变体优先于旧节点声明色；未指定变体保留声明色回退，不可用状态先统一使用禁用文字角色。

use super::super::resolved_state_for_node;
use super::helpers::{declared_color, is_unavailable_status_state};
use super::model::{
    WorkbenchStatusSignalKind, WorkbenchStatusSignalStyle, WORKBENCH_DIAGNOSTIC_SIGNAL_VARIANT,
    WORKBENCH_SEMANTIC_STATUS_SIGNAL_VARIANT,
};
use super::palette::{workbench_status_control_palette, WorkbenchStatusControlPalette};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

/// 使用调用端识别的信号种类及节点变体选择图标、文字配方。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_status_signal_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchStatusSignalKind,
) -> WorkbenchStatusSignalStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Generic);
    let palette = workbench_status_control_palette();

    WorkbenchStatusSignalStyle {
        icon_fill: status_signal_icon_fill(node, kind, state, &palette),
        text: status_signal_text_color(node, kind, state, &palette),
        state,
    }
}

fn status_signal_icon_fill(
    node: &TemplatePaneNodeData,
    kind: WorkbenchStatusSignalKind,
    state: UiPainterResolvedState,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    if is_unavailable_status_state(state) {
        return palette.text_disabled;
    }
    if node.component_variant.as_str() == WORKBENCH_DIAGNOSTIC_SIGNAL_VARIANT {
        return diagnostic_signal_color(kind, palette);
    }
    if node.component_variant.as_str() == WORKBENCH_SEMANTIC_STATUS_SIGNAL_VARIANT {
        return semantic_status_signal_icon_color(kind, palette);
    }
    if let Some(color) = declared_color(node.label_color) {
        return color;
    }
    semantic_status_signal_icon_color(kind, palette)
}

fn semantic_status_signal_icon_color(
    kind: WorkbenchStatusSignalKind,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    match kind {
        WorkbenchStatusSignalKind::Ready => palette.success,
        WorkbenchStatusSignalKind::Success => palette.no_errors_fill,
        WorkbenchStatusSignalKind::Warning => palette.warning,
        WorkbenchStatusSignalKind::Info => palette.info,
        WorkbenchStatusSignalKind::Error => palette.error,
    }
}

fn status_signal_text_color(
    node: &TemplatePaneNodeData,
    kind: WorkbenchStatusSignalKind,
    state: UiPainterResolvedState,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    if is_unavailable_status_state(state) {
        return palette.text_disabled;
    }
    if node.component_variant.as_str() == WORKBENCH_DIAGNOSTIC_SIGNAL_VARIANT {
        return diagnostic_signal_color(kind, palette);
    }
    if node.component_variant.as_str() == WORKBENCH_SEMANTIC_STATUS_SIGNAL_VARIANT {
        return semantic_status_signal_text_color(kind, palette);
    }
    if let Some(color) = declared_color(node.value_color) {
        return color;
    }
    semantic_status_signal_text_color(kind, palette)
}

fn semantic_status_signal_text_color(
    kind: WorkbenchStatusSignalKind,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    match kind {
        WorkbenchStatusSignalKind::Ready => palette.text,
        WorkbenchStatusSignalKind::Success
        | WorkbenchStatusSignalKind::Warning
        | WorkbenchStatusSignalKind::Info
        | WorkbenchStatusSignalKind::Error => palette.text_muted,
    }
}

fn diagnostic_signal_color(
    kind: WorkbenchStatusSignalKind,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    match kind {
        WorkbenchStatusSignalKind::Ready | WorkbenchStatusSignalKind::Success => palette.success,
        WorkbenchStatusSignalKind::Warning => palette.warning,
        WorkbenchStatusSignalKind::Info => palette.text_muted,
        WorkbenchStatusSignalKind::Error => palette.error,
    }
}
