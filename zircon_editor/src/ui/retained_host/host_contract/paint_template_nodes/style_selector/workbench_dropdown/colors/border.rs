use super::super::palette::{WORKBENCH_DROPDOWN_BORDER, WORKBENCH_DROPDOWN_FOCUS_BORDER};
use super::super::state::is_unavailable_dropdown_state;
use super::declared::declared_style_color;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_border(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let color = if is_unavailable_dropdown_state(state) {
        PALETTE.border_disabled
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else {
        match state {
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open => WORKBENCH_DROPDOWN_FOCUS_BORDER,
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => PALETTE.border,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                PALETTE.border_disabled
            }
            UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => WORKBENCH_DROPDOWN_BORDER,
        }
    };
    if is_unavailable_dropdown_state(state) {
        color
    } else {
        declared_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(color)
    }
}
