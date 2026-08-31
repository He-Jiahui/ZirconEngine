use crate::ui::retained_host::host_contract::data::{
    HostPaneInteractionStateData, HostWindowPresentationData,
};

use super::super::super::PanePointerRoute;
use super::super::mode::PaneRouteMode;
use super::floating::{route_floating_window_pane, FloatingPaneRoute};
use super::local::route_local_dock_pane;

pub(in crate::ui::retained_host::host_contract) fn route_pointer_to_pane<'a>(
    presentation: &'a HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute<'a>> {
    route_pointer_to_pane_with_mode(presentation, interaction, x, y, PaneRouteMode::Default)
}

pub(in crate::ui::retained_host::host_contract) fn route_pointer_move_to_pane<'a>(
    presentation: &'a HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute<'a>> {
    route_pointer_to_pane_with_mode(presentation, interaction, x, y, PaneRouteMode::PointerMove)
}

pub(in crate::ui::retained_host::host_contract) fn route_pointer_scroll_to_pane<'a>(
    presentation: &'a HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute<'a>> {
    route_pointer_to_pane_with_mode(
        presentation,
        interaction,
        x,
        y,
        PaneRouteMode::PointerScroll,
    )
}

fn route_pointer_to_pane_with_mode<'a>(
    presentation: &'a HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
    x: f32,
    y: f32,
    mode: PaneRouteMode,
) -> Option<PanePointerRoute<'a>> {
    let console_scroll_px = interaction.console_scroll_px;
    match route_floating_window_pane(presentation, x, y, mode, console_scroll_px) {
        FloatingPaneRoute::Routed(route) => Some(route),
        FloatingPaneRoute::Occluded => None,
        FloatingPaneRoute::Miss => {
            route_local_dock_pane(presentation, x, y, mode, console_scroll_px)
        }
    }
}

#[cfg(test)]
#[path = "route/tests.rs"]
mod tests;
