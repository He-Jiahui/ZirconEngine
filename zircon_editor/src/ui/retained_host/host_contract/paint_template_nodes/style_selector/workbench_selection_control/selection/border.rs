use super::super::colors::declared_style_border;
use super::super::model::WorkbenchSelectionControlKind;
use super::super::palette::WorkbenchSelectionControlPalette;
use super::super::state::{is_hot, is_unavailable_selection_state};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(super) fn control_border(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSelectionControlKind,
    state: UiPainterResolvedState,
    checked: bool,
    palette: WorkbenchSelectionControlPalette,
) -> [u8; 4] {
    if is_unavailable_selection_state(state) {
        return palette.border_disabled;
    }
    match kind {
        WorkbenchSelectionControlKind::Checkbox => {
            if is_hot(state) {
                palette.focus_ring
            } else if checked {
                palette.accent
            } else {
                declared_style_border(node).unwrap_or(palette.mark_idle_border)
            }
        }
        WorkbenchSelectionControlKind::Radio => {
            if is_hot(state) {
                palette.focus_ring
            } else if checked {
                palette.radio_checked_border
            } else {
                declared_style_border(node).unwrap_or(palette.mark_idle_border)
            }
        }
        WorkbenchSelectionControlKind::Toggle => {
            if is_hot(state) {
                palette.focus_ring
            } else if checked {
                palette.toggle_checked_border
            } else {
                declared_style_border(node).unwrap_or(palette.border)
            }
        }
    }
}
