use super::super::palette::{WORKBENCH_DROPDOWN_PLACEHOLDER, WORKBENCH_DROPDOWN_TEXT};
use super::super::state::is_unavailable_dropdown_state;
use super::declared::declared_color;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_text(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    label_is_placeholder: bool,
) -> [u8; 4] {
    if is_unavailable_dropdown_state(state) {
        PALETTE.text_disabled
    } else if let Some(color) = declared_color(node.value_color) {
        color
    } else if label_is_placeholder {
        WORKBENCH_DROPDOWN_PLACEHOLDER
    } else {
        WORKBENCH_DROPDOWN_TEXT
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_dropdown_state(state) {
        PALETTE.text_disabled
    } else if let Some(color) = declared_color(node.icon_color) {
        color
    } else if matches!(
        state,
        UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open
    ) {
        PALETTE.focus_ring
    } else {
        PALETTE.text_muted
    }
}
