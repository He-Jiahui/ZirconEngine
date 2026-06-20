use super::colors::declared_style_color;
use super::palette::{
    WORKBENCH_TEXT_FIELD_BORDER, WORKBENCH_TEXT_FIELD_DISABLED_BORDER,
    WORKBENCH_TEXT_FIELD_DISABLED_SURFACE, WORKBENCH_TEXT_FIELD_FOCUSED_BORDER,
    WORKBENCH_TEXT_FIELD_FOCUSED_SURFACE, WORKBENCH_TEXT_FIELD_HOVER_SURFACE,
    WORKBENCH_TEXT_FIELD_SURFACE,
};
use super::state::is_unavailable_text_field_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_surface(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let color = match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            WORKBENCH_TEXT_FIELD_DISABLED_SURFACE
        }
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open => WORKBENCH_TEXT_FIELD_FOCUSED_SURFACE,
        UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => WORKBENCH_TEXT_FIELD_HOVER_SURFACE,
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => WORKBENCH_TEXT_FIELD_SURFACE,
    };
    if is_unavailable_text_field_state(state) {
        color
    } else {
        declared_style_color(node.button_style.element.background_color.as_ref()).unwrap_or(color)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_border(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let color = if is_unavailable_text_field_state(state) {
        WORKBENCH_TEXT_FIELD_DISABLED_BORDER
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else {
        match state {
            UiPainterResolvedState::Pressed => PALETTE.focus_ring,
            UiPainterResolvedState::Focused | UiPainterResolvedState::Open => {
                WORKBENCH_TEXT_FIELD_FOCUSED_BORDER
            }
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => PALETTE.border,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                WORKBENCH_TEXT_FIELD_DISABLED_BORDER
            }
            UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => WORKBENCH_TEXT_FIELD_BORDER,
        }
    };
    if is_unavailable_text_field_state(state) {
        color
    } else {
        declared_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(color)
    }
}
