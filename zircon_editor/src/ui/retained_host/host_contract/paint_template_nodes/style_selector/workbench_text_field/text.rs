use super::palette::{
    WORKBENCH_TEXT_FIELD_DISABLED_TEXT, WORKBENCH_TEXT_FIELD_PLACEHOLDER, WORKBENCH_TEXT_FIELD_TEXT,
};
use super::state::is_unavailable_text_field_state;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_text(
    state: UiPainterResolvedState,
    label_is_placeholder: bool,
) -> [u8; 4] {
    if is_unavailable_text_field_state(state) {
        WORKBENCH_TEXT_FIELD_DISABLED_TEXT
    } else if label_is_placeholder {
        WORKBENCH_TEXT_FIELD_PLACEHOLDER
    } else {
        WORKBENCH_TEXT_FIELD_TEXT
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_stepper(
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_text_field_state(state) {
        PALETTE.text_disabled
    } else {
        PALETTE.text_muted
    }
}
