use super::super::colors::declared_style_background;
use super::super::model::WorkbenchSelectionControlKind;
use super::super::palette::WorkbenchSelectionControlPalette;
use super::super::state::{is_hot, is_unavailable_selection_state};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(super) fn control_surface(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSelectionControlKind,
    state: UiPainterResolvedState,
    checked: bool,
    palette: WorkbenchSelectionControlPalette,
) -> [u8; 4] {
    if is_unavailable_selection_state(state) {
        return palette.surface_disabled;
    }
    match kind {
        WorkbenchSelectionControlKind::Checkbox => {
            if checked {
                palette.checkbox_checked_fill
            } else {
                declared_style_background(node).unwrap_or(palette.mark_idle_fill)
            }
        }
        WorkbenchSelectionControlKind::Radio => {
            if checked {
                palette.radio_checked_fill
            } else {
                declared_style_background(node).unwrap_or(palette.mark_idle_fill)
            }
        }
        WorkbenchSelectionControlKind::Toggle => {
            if checked {
                palette.toggle_checked_surface
            } else if state == UiPainterResolvedState::Pressed {
                palette.toggle_pressed_surface
            } else if is_hot(state) {
                palette.toggle_hover_surface
            } else {
                declared_style_background(node).unwrap_or(palette.toggle_track)
            }
        }
    }
}
