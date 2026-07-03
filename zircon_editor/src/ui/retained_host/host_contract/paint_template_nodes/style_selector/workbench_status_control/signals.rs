use super::super::resolved_state_for_node;
use super::helpers::{declared_color, is_unavailable_status_state};
use super::model::{WorkbenchStatusSignalKind, WorkbenchStatusSignalStyle};
use super::palette::{workbench_status_control_palette, WorkbenchStatusControlPalette};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

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
    if let Some(color) = declared_color(node.label_color) {
        return color;
    }
    match kind {
        WorkbenchStatusSignalKind::Ready => palette.success,
        WorkbenchStatusSignalKind::Success => palette.no_errors_fill,
        WorkbenchStatusSignalKind::Warning => palette.warning,
        WorkbenchStatusSignalKind::Info => palette.info,
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
    if let Some(color) = declared_color(node.value_color) {
        return color;
    }
    match kind {
        WorkbenchStatusSignalKind::Ready => palette.text,
        WorkbenchStatusSignalKind::Success
        | WorkbenchStatusSignalKind::Warning
        | WorkbenchStatusSignalKind::Info => palette.text_muted,
    }
}
