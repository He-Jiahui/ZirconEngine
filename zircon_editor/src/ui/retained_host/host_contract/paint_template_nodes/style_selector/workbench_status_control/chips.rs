use super::super::resolved_state_for_node;
use super::helpers::{declared_color, is_unavailable_status_state};
use super::model::WorkbenchStatusChipStyle;
use super::palette::WORKBENCH_STATUS_FLAT_TRANSPARENT;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_status_chip_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchStatusChipStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Generic);

    WorkbenchStatusChipStyle {
        background: status_chip_background(state),
        border: status_chip_border(state),
        text: status_chip_text_color(node, state),
        state,
    }
}

fn status_chip_background(state: UiPainterResolvedState) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            PALETTE.surface_disabled
        }
        UiPainterResolvedState::Pressed => PALETTE.surface_pressed,
        UiPainterResolvedState::Selected | UiPainterResolvedState::Checked => {
            PALETTE.surface_selected
        }
        UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => PALETTE.surface_hover,
        UiPainterResolvedState::Normal => WORKBENCH_STATUS_FLAT_TRANSPARENT,
    }
}

fn status_chip_border(state: UiPainterResolvedState) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            PALETTE.border_disabled
        }
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked => PALETTE.focus_ring,
        UiPainterResolvedState::Hovered | UiPainterResolvedState::Normal => {
            WORKBENCH_STATUS_FLAT_TRANSPARENT
        }
    }
}

fn status_chip_text_color(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_status_state(state) {
        PALETTE.text_disabled
    } else {
        declared_color(node.value_color).unwrap_or(PALETTE.text_muted)
    }
}
