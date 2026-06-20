use super::state::is_unavailable_list_row_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_text_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    if is_unavailable_list_row_state(state) {
        PALETTE.text_disabled
    } else if let Some(color) = declared_color(node.value_color) {
        color
    } else if marked {
        PALETTE.text
    } else {
        PALETTE.text_muted
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn list_row_adornment_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    if is_unavailable_list_row_state(state) {
        PALETTE.text_disabled
    } else if let Some(color) = declared_color(node.icon_color) {
        color
    } else if marked {
        PALETTE.focus_ring
    } else {
        PALETTE.text_muted
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}
