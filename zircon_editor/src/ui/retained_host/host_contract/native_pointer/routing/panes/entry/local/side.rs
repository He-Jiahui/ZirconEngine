use crate::ui::retained_host::host_contract::data::HostSideDockSurfaceData;

use super::super::super::super::PanePointerRoute;
use super::super::super::super::geometry::side_dock_content_frame;
use super::super::super::mode::PaneRouteMode;
use super::super::super::pane::pane_route_from_pane;

pub(super) fn route_side_dock_pane(
    dock: &HostSideDockSurfaceData,
    x: f32,
    y: f32,
    mode: PaneRouteMode,
) -> Option<PanePointerRoute> {
    let content = side_dock_content_frame(dock);
    pane_route_from_pane(
        &dock.pane,
        &content,
        x,
        y,
        Some(dock.surface_key.as_str()),
        mode,
    )
}
