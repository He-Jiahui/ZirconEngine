use super::super::resolved_state_for_node;
use super::helpers::{
    declared_color, is_unavailable_status_state, status_node_is_hot, status_node_is_selected,
};
use super::model::WorkbenchStatusChipStyle;
use super::palette::{workbench_status_control_palette, WorkbenchStatusControlPalette};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_status_chip_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchStatusChipStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Generic);
    let palette = workbench_status_control_palette();

    WorkbenchStatusChipStyle {
        background: status_chip_background(node, state, &palette),
        border: status_chip_border(state, &palette),
        label_text: status_chip_label_text_color(node, state, &palette),
        value_text: status_chip_value_text_color(node, state, &palette),
        state,
    }
}

fn status_chip_background(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            palette.surface_disabled
        }
        UiPainterResolvedState::Pressed => palette.surface_pressed,
        UiPainterResolvedState::Selected | UiPainterResolvedState::Checked => {
            palette.surface_selected
        }
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

fn status_chip_border(
    state: UiPainterResolvedState,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            palette.border_disabled
        }
        UiPainterResolvedState::Focused | UiPainterResolvedState::DropHovered => palette.focus_ring,
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked => palette.border,
        UiPainterResolvedState::Hovered | UiPainterResolvedState::Normal => {
            palette.flat_transparent
        }
    }
}

fn status_chip_label_text_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    if is_unavailable_status_state(state) {
        palette.text_disabled
    } else {
        declared_color(node.label_color).unwrap_or(palette.text_muted)
    }
}

fn status_chip_value_text_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    palette: &WorkbenchStatusControlPalette,
) -> [u8; 4] {
    if is_unavailable_status_state(state) {
        palette.text_disabled
    } else {
        declared_color(node.value_color).unwrap_or(palette.text)
    }
}
