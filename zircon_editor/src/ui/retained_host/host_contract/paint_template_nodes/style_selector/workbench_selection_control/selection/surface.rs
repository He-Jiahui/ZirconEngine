use super::super::colors::declared_style_background;
use super::super::model::WorkbenchSelectionControlKind;
use super::super::palette::{
    WORKBENCH_CHECKBOX_CHECKED_FILL, WORKBENCH_RADIO_CHECKED_FILL,
    WORKBENCH_SELECTION_MARK_IDLE_FILL,
};
use super::super::state::{is_hot, is_unavailable_selection_state};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(super) fn control_surface(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSelectionControlKind,
    state: UiPainterResolvedState,
    checked: bool,
) -> [u8; 4] {
    if is_unavailable_selection_state(state) {
        return PALETTE.surface_disabled;
    }
    match kind {
        WorkbenchSelectionControlKind::Checkbox => {
            if checked {
                declared_style_background(node).unwrap_or(WORKBENCH_CHECKBOX_CHECKED_FILL)
            } else {
                declared_style_background(node).unwrap_or(WORKBENCH_SELECTION_MARK_IDLE_FILL)
            }
        }
        WorkbenchSelectionControlKind::Radio => {
            if checked {
                declared_style_background(node).unwrap_or(WORKBENCH_RADIO_CHECKED_FILL)
            } else {
                declared_style_background(node).unwrap_or(WORKBENCH_SELECTION_MARK_IDLE_FILL)
            }
        }
        WorkbenchSelectionControlKind::Toggle => {
            if checked {
                declared_style_background(node).unwrap_or(PALETTE.accent)
            } else if state == UiPainterResolvedState::Pressed {
                PALETTE.surface_pressed
            } else if is_hot(state) {
                PALETTE.surface_hover
            } else {
                declared_style_background(node).unwrap_or(PALETTE.track)
            }
        }
    }
}
