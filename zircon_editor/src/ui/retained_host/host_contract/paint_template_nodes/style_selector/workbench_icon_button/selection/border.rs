use super::super::model::WorkbenchIconButtonContext;
use super::super::palette::workbench_icon_button_palette;
use super::super::state::is_unavailable_icon_button_state;
use super::declared::declared_icon_button_border;
use super::toolbar_chrome::icon_toolbar_normal_chrome_enabled;
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
    match state {
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => Some(palette.focus_ring),
        UiPainterResolvedState::Hovered => Some(palette.border),
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            (context == WorkbenchIconButtonContext::Panel).then_some(palette.border_disabled)
        }
        UiPainterResolvedState::Normal => {
            if context == WorkbenchIconButtonContext::Panel {
                declared_icon_button_border(node).or(Some(palette.panel_border))
            } else if context == WorkbenchIconButtonContext::Toolbar
                && icon_toolbar_normal_chrome_enabled(node)
            {
                Some(palette.border)
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
    if context == WorkbenchIconButtonContext::Panel
        || (context == WorkbenchIconButtonContext::Toolbar
            && icon_toolbar_normal_chrome_enabled(node))
        || state != UiPainterResolvedState::Normal
    {
        metrics.border_width
    } else {
        0.0
    }
}
