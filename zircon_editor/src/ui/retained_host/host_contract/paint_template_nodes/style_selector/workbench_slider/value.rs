use super::palette::{SLIDER_VALUE_BORDER, SLIDER_VALUE_SURFACE};
use super::state::is_unavailable_slider_state;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_value_surface(
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        PALETTE.surface_disabled
    } else {
        SLIDER_VALUE_SURFACE
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_value_border(
    state: UiPainterResolvedState,
    fill: [u8; 4],
) -> [u8; 4] {
    if is_unavailable_slider_state(state) {
        return PALETTE.border_disabled;
    }
    if matches!(
        state,
        UiPainterResolvedState::Focused | UiPainterResolvedState::Pressed
    ) {
        fill
    } else {
        SLIDER_VALUE_BORDER
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_range_value_border(
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_slider_state(state) {
        PALETTE.border_disabled
    } else {
        SLIDER_VALUE_BORDER
    }
}
