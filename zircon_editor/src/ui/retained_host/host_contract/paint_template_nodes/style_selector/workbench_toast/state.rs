use super::model::WorkbenchToastStyle;
use super::palette::{
    toast_normal_style, WORKBENCH_TOAST_HOVER_SURFACE, WORKBENCH_TOAST_PRESSED_SURFACE,
};
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_state_style(
    state: UiPainterResolvedState,
) -> WorkbenchToastStyle {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => WorkbenchToastStyle {
            surface: PALETTE.surface_disabled,
            border: PALETTE.border_disabled,
            text: PALETTE.text_disabled,
            mark: PALETTE.text_disabled,
            action: PALETTE.text_disabled,
            close: PALETTE.text_disabled,
            state,
        },
        UiPainterResolvedState::Pressed => {
            let mut style = toast_normal_style(state);
            style.surface = WORKBENCH_TOAST_PRESSED_SURFACE;
            style.border = PALETTE.focus_ring;
            style.action = PALETTE.focus_ring;
            style
        }
        UiPainterResolvedState::Focused | UiPainterResolvedState::Open => {
            let mut style = toast_normal_style(state);
            style.border = PALETTE.focus_ring;
            style.action = PALETTE.focus_ring;
            style
        }
        UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => {
            let mut style = toast_normal_style(state);
            style.surface = WORKBENCH_TOAST_HOVER_SURFACE;
            style.border = PALETTE.accent_soft;
            style
        }
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => toast_normal_style(state),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_toast_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}
