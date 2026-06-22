use crate::ui::retained_host::host_contract::data::HostDocumentDockSurfaceData;

use super::super::super::super::geometry::translated;
use super::super::super::super::PanePointerRoute;
use super::super::super::mode::PaneRouteMode;
use super::super::super::pane::pane_route_from_pane;

pub(super) fn route_document_dock_pane(
    dock: &HostDocumentDockSurfaceData,
    x: f32,
    y: f32,
    mode: PaneRouteMode,
) -> Option<PanePointerRoute> {
    let content = translated(
        &dock.content_frame,
        dock.region_frame.x,
        dock.region_frame.y,
    );
    pane_route_from_pane(
        &dock.pane,
        &content,
        x,
        y,
        Some(dock.surface_key.as_str()),
        mode,
    )
}
