use super::super::resolved_state_for_node;
use super::model::WorkbenchStatusIconButtonStyle;
use super::palette::{
    WORKBENCH_STATUS_FLAT_TRANSPARENT, WORKBENCH_STATUS_ICON_COLOR, WORKBENCH_STATUS_ICON_MUTED,
};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_status_icon_button_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchStatusIconButtonStyle {
    let state =
        resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::IconButton);

    WorkbenchStatusIconButtonStyle {
        background: status_icon_button_background(state),
        border: status_icon_button_border(state),
        glyph: status_icon_glyph_color(state),
        state,
    }
}

fn status_icon_button_background(state: UiPainterResolvedState) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            PALETTE.surface_disabled
        }
        UiPainterResolvedState::Selected | UiPainterResolvedState::Checked => {
            PALETTE.surface_selected
        }
        UiPainterResolvedState::Pressed => PALETTE.surface_pressed,
        UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => PALETTE.surface_hover,
        UiPainterResolvedState::Normal => WORKBENCH_STATUS_FLAT_TRANSPARENT,
    }
}

fn status_icon_button_border(state: UiPainterResolvedState) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            PALETTE.border_disabled
        }
        UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => PALETTE.focus_ring,
        UiPainterResolvedState::Hovered | UiPainterResolvedState::Normal => {
            WORKBENCH_STATUS_FLAT_TRANSPARENT
        }
    }
}

fn status_icon_glyph_color(state: UiPainterResolvedState) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => PALETTE.text_disabled,
        UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => PALETTE.focus_ring,
        UiPainterResolvedState::Hovered => WORKBENCH_STATUS_ICON_COLOR,
        UiPainterResolvedState::Normal => WORKBENCH_STATUS_ICON_MUTED,
    }
}
