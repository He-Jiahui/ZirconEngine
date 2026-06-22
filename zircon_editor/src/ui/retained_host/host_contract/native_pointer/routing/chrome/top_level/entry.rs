use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;

use super::super::super::ChromePointerRoute;
use super::super::floating::route_floating_window_header;
use super::super::tabs::route_host_page_tabs;
use super::docked::route_document_dock_tabs;
use super::drawers::route_drawer_headers;
use super::resize::route_resize_splitters;
use super::side_rails::route_side_activity_rails;

pub(in crate::ui::retained_host::host_contract) fn route_top_level_chrome(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let scene = &presentation.host_scene_data;
    if let Some(route) = route_resize_splitters(&scene.resize_layer, x, y) {
        return Some(route);
    }

    if let Some(route) = route_document_dock_tabs(presentation, x, y) {
        return Some(route);
    }
    if let Some(route) = route_side_activity_rails(presentation, x, y) {
        return Some(route);
    }
    if let Some(route) = route_drawer_headers(scene, x, y) {
        return Some(route);
    }
    if let Some(route) = route_host_page_tabs(&scene.page_chrome.tab_frames, x, y) {
        return Some(route);
    }
    if let Some(route) = route_floating_window_header(&scene.floating_layer, x, y) {
        return Some(route);
    }

    None
}
