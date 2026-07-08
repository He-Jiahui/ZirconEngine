use super::super::resolved_state_for_node;
use super::helpers::{status_node_is_hot, status_node_is_selected, status_node_uses_active_glyph};
use super::model::WorkbenchStatusIconButtonStyle;
use super::palette::{workbench_status_control_palette, WorkbenchStatusControlPalette};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_status_icon_button_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchStatusIconButtonStyle {
    let state =
        resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::IconButton);
    let palette = workbench_status_control_palette();

    WorkbenchStatusIconButtonStyle {
        background: status_icon_button_background(node, state, &palette),
        border: status_icon_button_border(state, &palette),
        glyph: status_icon_glyph_color(node, state, &palette),
        state,
    }
}

fn status_icon_button_background(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            palette.surface_disabled
        }
        UiPainterResolvedState::Selected | UiPainterResolvedState::Checked => {
            palette.surface_selected
        }
        UiPainterResolvedState::Pressed => palette.surface_pressed,
        UiPainterResolvedState::Focused => {
            if status_node_is_selected(node) {
                palette.surface_selected
            } else if status_node_is_hot(node) {
                palette.surface_hover
            } else {
                palette.flat_transparent
            }
        }
        UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => palette.surface_hover,
        UiPainterResolvedState::Normal => palette.flat_transparent,
    }
}

fn status_icon_button_border(
    state: UiPainterResolvedState,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            palette.border_disabled
        }
        UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => palette.focus_ring,
        UiPainterResolvedState::Hovered | UiPainterResolvedState::Normal => {
            palette.flat_transparent
        }
    }
}

fn status_icon_glyph_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => palette.text_disabled,
        UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => palette.focus_ring,
        UiPainterResolvedState::Focused => {
            if status_node_uses_active_glyph(node) {
                palette.focus_ring
            } else if node.hovered {
                palette.icon_color
            } else {
                palette.icon_muted
            }
        }
        UiPainterResolvedState::Hovered => palette.icon_color,
        UiPainterResolvedState::Normal => palette.icon_muted,
    }
}
