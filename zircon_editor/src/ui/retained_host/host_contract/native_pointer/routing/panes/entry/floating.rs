use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;

use super::super::super::PanePointerRoute;
use super::super::super::geometry::{contains, floating_window_content_frame};
use super::super::mode::PaneRouteMode;
use super::super::pane::pane_route_from_pane;

pub(super) enum FloatingPaneRoute {
    Miss,
    Occluded,
    Routed(PanePointerRoute),
}

pub(super) fn route_floating_window_pane(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
    mode: PaneRouteMode,
) -> FloatingPaneRoute {
    let scene = &presentation.host_scene_data;
    for row in (0..scene.floating_layer.floating_windows.row_count()).rev() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        let content = floating_window_content_frame(&window.frame, &window.header_frame);
        if contains(&content, x, y) {
            return match pane_route_from_pane(
                &window.active_pane,
                &content,
                x,
                y,
                Some(window.window_id.as_str()),
                mode,
            ) {
                Some(route) => FloatingPaneRoute::Routed(route),
                None => FloatingPaneRoute::Occluded,
            };
        }
    }
    FloatingPaneRoute::Miss
}
