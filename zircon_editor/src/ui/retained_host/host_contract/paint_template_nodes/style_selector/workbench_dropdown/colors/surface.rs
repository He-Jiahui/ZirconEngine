use super::super::palette::{
    WORKBENCH_DROPDOWN_DISABLED_SURFACE, WORKBENCH_DROPDOWN_HOVER_SURFACE,
    WORKBENCH_DROPDOWN_OPEN_SURFACE, WORKBENCH_DROPDOWN_SURFACE,
};
use super::super::state::is_unavailable_dropdown_state;
use super::declared::declared_style_color;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_surface(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let color = match state {
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open => WORKBENCH_DROPDOWN_OPEN_SURFACE,
        UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => WORKBENCH_DROPDOWN_HOVER_SURFACE,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            WORKBENCH_DROPDOWN_DISABLED_SURFACE
        }
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => WORKBENCH_DROPDOWN_SURFACE,
    };
    if is_unavailable_dropdown_state(state) {
        color
    } else {
        declared_style_color(node.button_style.element.background_color.as_ref()).unwrap_or(color)
    }
}
