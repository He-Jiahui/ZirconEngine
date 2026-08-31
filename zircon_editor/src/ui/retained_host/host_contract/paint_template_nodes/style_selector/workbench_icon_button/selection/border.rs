use super::super::model::WorkbenchIconButtonContext;
use super::super::palette::workbench_icon_button_palette;
use super::super::state::{icon_button_node_is_hot, is_unavailable_icon_button_state};
use super::declared::declared_icon_button_border;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_border(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    state: UiPainterResolvedState,
    danger: bool,
) -> Option<[u8; 4]> {
    let palette = workbench_icon_button_palette();
    if is_unavailable_icon_button_state(state) {
        return (context == WorkbenchIconButtonContext::Panel).then_some(palette.border_disabled);
    }
    if danger && context == WorkbenchIconButtonContext::Panel {
        return declared_icon_button_border(node).or(Some(palette.error));
    }
    if context == WorkbenchIconButtonContext::Toolbar {
        // Starship's ToolbarButton selection is a rounded fill, not an input-like
        // outline. Keep an outline exclusively for keyboard focus visibility.
        return (state == UiPainterResolvedState::Focused && !icon_button_node_is_hot(node))
            .then_some(palette.focus_ring);
    }
    match state {
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging => Some(palette.panel_border),
        UiPainterResolvedState::Focused | UiPainterResolvedState::DropHovered => {
            Some(palette.focus_ring)
        }
        UiPainterResolvedState::Hovered => Some(palette.border),
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            (context == WorkbenchIconButtonContext::Panel).then_some(palette.border_disabled)
        }
        UiPainterResolvedState::Normal => {
            if context == WorkbenchIconButtonContext::Panel {
                declared_icon_button_border(node).or(Some(palette.panel_border))
            } else {
                None
            }
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_border_width(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    state: UiPainterResolvedState,
) -> f32 {
    icon_border_width_from_host(node, context, state, current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_border_width_from_host(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    state: UiPainterResolvedState,
    metrics: HostControlMetrics,
) -> f32 {
    if context == WorkbenchIconButtonContext::Toolbar {
        return (state == UiPainterResolvedState::Focused && !icon_button_node_is_hot(node))
            .then_some(metrics.border_width)
            .unwrap_or(0.0);
    }
    if context == WorkbenchIconButtonContext::Panel || state != UiPainterResolvedState::Normal {
        metrics.border_width
    } else {
        0.0
    }
}
