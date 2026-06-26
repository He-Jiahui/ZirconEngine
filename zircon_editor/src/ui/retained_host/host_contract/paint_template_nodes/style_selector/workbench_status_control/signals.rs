use super::super::resolved_state_for_node;
use super::helpers::{declared_color, is_unavailable_status_state};
use super::model::{WorkbenchStatusSignalKind, WorkbenchStatusSignalStyle};
use super::palette::WORKBENCH_STATUS_NO_ERRORS_FILL;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_status_signal_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchStatusSignalKind,
) -> WorkbenchStatusSignalStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Generic);

    WorkbenchStatusSignalStyle {
        icon_fill: status_signal_icon_fill(node, kind, state),
        text: status_signal_text_color(node, kind, state),
        state,
    }
}

fn status_signal_icon_fill(
    node: &TemplatePaneNodeData,
    kind: WorkbenchStatusSignalKind,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_status_state(state) {
        return PALETTE.text_disabled;
    }
    if let Some(color) = declared_color(node.label_color) {
        return color;
    }
    match kind {
        WorkbenchStatusSignalKind::Ready => PALETTE.success,
        WorkbenchStatusSignalKind::Success => WORKBENCH_STATUS_NO_ERRORS_FILL,
        WorkbenchStatusSignalKind::Warning => PALETTE.warning,
        WorkbenchStatusSignalKind::Info => PALETTE.info,
    }
}

fn status_signal_text_color(
    node: &TemplatePaneNodeData,
    kind: WorkbenchStatusSignalKind,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_status_state(state) {
        return PALETTE.text_disabled;
    }
    if let Some(color) = declared_color(node.value_color) {
        return color;
    }
    match kind {
        WorkbenchStatusSignalKind::Ready => PALETTE.text,
        WorkbenchStatusSignalKind::Success
        | WorkbenchStatusSignalKind::Warning
        | WorkbenchStatusSignalKind::Info => PALETTE.text_muted,
    }
}
