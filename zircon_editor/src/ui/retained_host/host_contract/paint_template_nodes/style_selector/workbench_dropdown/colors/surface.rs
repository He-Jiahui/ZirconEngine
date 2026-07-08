use super::super::palette::workbench_dropdown_palette;
use super::super::state::{
    dropdown_node_is_hot, dropdown_node_is_open, is_unavailable_dropdown_state,
};
use super::declared::declared_style_color;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_surface(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_dropdown_palette();
    let color = match state {
        UiPainterResolvedState::Pressed | UiPainterResolvedState::Open => palette.open_surface,
        UiPainterResolvedState::Focused => {
            if dropdown_node_is_open(node) {
                palette.open_surface
            } else if dropdown_node_is_hot(node) {
                palette.hover_surface
            } else {
                palette.surface
            }
        }
        UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => palette.hover_surface,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            palette.disabled_surface
        }
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => palette.surface,
    };
    if is_unavailable_dropdown_state(state) {
        color
    } else {
        declared_style_color(node.button_style.element.background_color.as_ref()).unwrap_or(color)
    }
}
