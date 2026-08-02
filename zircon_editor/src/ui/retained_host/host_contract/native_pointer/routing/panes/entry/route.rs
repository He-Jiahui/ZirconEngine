use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;

use super::super::super::PanePointerRoute;
use super::super::mode::PaneRouteMode;
use super::floating::{FloatingPaneRoute, route_floating_window_pane};
use super::local::route_local_dock_pane;

pub(in crate::ui::retained_host::host_contract) fn route_pointer_to_pane(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute> {
    route_pointer_to_pane_with_mode(presentation, x, y, PaneRouteMode::Default)
}

pub(in crate::ui::retained_host::host_contract) fn route_pointer_move_to_pane(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute> {
    route_pointer_to_pane_with_mode(presentation, x, y, PaneRouteMode::PointerMove)
}

pub(in crate::ui::retained_host::host_contract) fn route_pointer_scroll_to_pane(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<PanePointerRoute> {
    route_pointer_to_pane_with_mode(presentation, x, y, PaneRouteMode::PointerScroll)
}

fn route_pointer_to_pane_with_mode(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
    mode: PaneRouteMode,
) -> Option<PanePointerRoute> {
    match route_floating_window_pane(presentation, x, y, mode) {
        FloatingPaneRoute::Routed(route) => Some(route),
        FloatingPaneRoute::Occluded => None,
        FloatingPaneRoute::Miss => route_local_dock_pane(presentation, x, y, mode),
    }
}

#[cfg(test)]
#[path = "route/tests.rs"]
mod tests;
