use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;

use super::super::super::geometry::{contains, floating_window_content_frame};
use super::super::super::PanePointerRoute;
use super::super::mode::PaneRouteMode;
use super::super::pane::pane_route_from_pane;

pub(super) enum FloatingPaneRoute<'a> {
    Miss,
    Occluded,
    Routed(PanePointerRoute<'a>),
}

pub(super) fn route_floating_window_pane<'a>(
    presentation: &'a HostWindowPresentationData,
    x: f32,
    y: f32,
    mode: PaneRouteMode,
    console_scroll_px: f32,
) -> FloatingPaneRoute<'a> {
    let scene = &presentation.host_scene_data;
    for window in scene.floating_layer.floating_windows.iter().rev() {
        let content = floating_window_content_frame(&window.frame, &window.header_frame);
        if contains(&content, x, y) {
            return match pane_route_from_pane(
                &window.active_pane,
                &content,
                x,
                y,
                Some(window.window_id.as_str()),
                mode,
                console_scroll_px,
            ) {
                Some(route) => FloatingPaneRoute::Routed(route),
                None => FloatingPaneRoute::Occluded,
            };
        }
    }
    FloatingPaneRoute::Miss
}
