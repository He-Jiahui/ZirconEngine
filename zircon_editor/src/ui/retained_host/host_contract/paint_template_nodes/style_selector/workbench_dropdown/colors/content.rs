use super::super::palette::workbench_dropdown_palette;
use super::super::state::is_unavailable_dropdown_state;
use super::declared::declared_color;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_text(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    label_is_placeholder: bool,
) -> [u8; 4] {
    let palette = workbench_dropdown_palette();
    if is_unavailable_dropdown_state(state) {
        palette.disabled_text
    } else if let Some(color) = declared_color(node.value_color) {
        color
    } else if label_is_placeholder {
        palette.placeholder
    } else {
        palette.text
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_chevron(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_dropdown_palette();
    if is_unavailable_dropdown_state(state) {
        palette.disabled_text
    } else if let Some(color) = declared_color(node.icon_color) {
        color
    } else if matches!(
        state,
        UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open
    ) {
        palette.active_chevron
    } else {
        palette.chevron
    }
}
