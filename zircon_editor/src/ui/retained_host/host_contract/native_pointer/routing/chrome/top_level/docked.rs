use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;

use super::super::super::{geometry::translated, ChromePointerRoute};
use super::super::tabs::{route_dock_overflow, route_document_tabs};

pub(super) fn route_document_dock_tabs(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let scene = &presentation.host_scene_data;
    for dock in scene.document_surfaces() {
        if let Some(route) = route_dock_overflow(
            dock.surface_key.as_str(),
            &dock.region_frame,
            &dock.overflow_frame,
            x,
            y,
        ) {
            return Some(route);
        }
        if let Some(route) = route_document_tabs(
            dock.surface_key.as_str(),
            &translated(&dock.header_frame, dock.region_frame.x, dock.region_frame.y),
            &dock.tab_frames,
            x,
            y,
        ) {
            return Some(route);
        }
    }
    None
}
