use super::super::model::WorkbenchIconButtonContext;
use super::super::palette::{workbench_icon_button_palette, WorkbenchIconButtonPalette};
use super::super::state::{
    icon_button_node_is_hot, icon_button_node_is_selected, is_unavailable_icon_button_state,
};
use super::declared::declared_icon_button_background;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_background(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    state: UiPainterResolvedState,
    danger: bool,
) -> Option<[u8; 4]> {
    let palette = workbench_icon_button_palette();
    if is_unavailable_icon_button_state(state) {
        return (context == WorkbenchIconButtonContext::Panel).then_some(palette.surface_disabled);
    }
    if danger && context == WorkbenchIconButtonContext::Panel {
        return Some(palette.error_container);
    }
    match state {
        UiPainterResolvedState::Selected | UiPainterResolvedState::Checked => {
            Some(palette.surface_selected)
        }
        UiPainterResolvedState::Pressed => Some(palette.surface_pressed),
        UiPainterResolvedState::Focused => {
            if icon_button_node_is_selected(node) {
                Some(palette.surface_selected)
            } else if icon_button_node_is_hot(node) {
                Some(palette.surface_hover)
            } else {
                normal_icon_background(node, context, palette)
            }
        }
        UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => Some(palette.surface_hover),
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            (context == WorkbenchIconButtonContext::Panel).then_some(palette.surface_disabled)
        }
        UiPainterResolvedState::Normal => normal_icon_background(node, context, palette),
    }
}

fn normal_icon_background(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    palette: WorkbenchIconButtonPalette,
) -> Option<[u8; 4]> {
    if context == WorkbenchIconButtonContext::Panel {
        declared_icon_button_background(node).or(Some(palette.panel_surface))
    } else {
        // FStarshipCoreStyle::ToolbarButton has no normal brush: toolbar glyphs
        // stay quiet until hover/pressed supplies a rounded selection surface.
        None
    }
}
