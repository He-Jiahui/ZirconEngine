use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;

use super::super::super::{geometry::translated, ChromePointerRoute};
use super::super::tabs::{route_dock_overflow, route_document_tabs};

pub(super) fn route_document_dock_tabs(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let scene = &presentation.host_scene_data;
    if let Some(route) = route_dock_overflow(
        scene.document_dock.surface_key.as_str(),
        &scene.document_dock.region_frame,
        &scene.document_dock.overflow_frame,
        x,
        y,
    ) {
        return Some(route);
    }
    route_document_tabs(
        "document",
        &translated(
            &scene.document_dock.header_frame,
            scene.document_dock.region_frame.x,
            scene.document_dock.region_frame.y,
        ),
        &scene.document_dock.tab_frames,
        x,
        y,
    )
}
