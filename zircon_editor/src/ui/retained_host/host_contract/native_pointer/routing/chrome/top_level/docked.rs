use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;

use super::super::super::{ChromePointerRoute, geometry::translated};
use super::super::tabs::route_document_tabs;

pub(super) fn route_document_dock_tabs(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let scene = &presentation.host_scene_data;
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
