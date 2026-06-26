use super::super::colors::declared_style_border;
use super::super::model::WorkbenchSelectionControlKind;
use super::super::palette::{WORKBENCH_RADIO_CHECKED_BORDER, WORKBENCH_SELECTION_MARK_IDLE_BORDER};
use super::super::state::{is_hot, is_unavailable_selection_state};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(super) fn control_border(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSelectionControlKind,
    state: UiPainterResolvedState,
    checked: bool,
) -> [u8; 4] {
    if is_unavailable_selection_state(state) {
        return PALETTE.border_disabled;
    }
    match kind {
        WorkbenchSelectionControlKind::Checkbox => {
            if is_hot(state) {
                PALETTE.focus_ring
            } else if checked {
                PALETTE.accent
            } else {
                declared_style_border(node).unwrap_or(WORKBENCH_SELECTION_MARK_IDLE_BORDER)
            }
        }
        WorkbenchSelectionControlKind::Radio => {
            if is_hot(state) {
                PALETTE.focus_ring
            } else if checked {
                WORKBENCH_RADIO_CHECKED_BORDER
            } else {
                declared_style_border(node).unwrap_or(WORKBENCH_SELECTION_MARK_IDLE_BORDER)
            }
        }
        WorkbenchSelectionControlKind::Toggle => {
            if checked || is_hot(state) {
                PALETTE.accent
            } else {
                declared_style_border(node).unwrap_or(PALETTE.border)
            }
        }
    }
}
