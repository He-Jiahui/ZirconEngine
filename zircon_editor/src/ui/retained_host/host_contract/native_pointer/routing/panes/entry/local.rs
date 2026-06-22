mod bottom;
mod document;
mod side;

use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;

use self::bottom::route_bottom_dock_pane;
use self::document::route_document_dock_pane;
use self::side::route_side_dock_pane;
use super::super::super::PanePointerRoute;
use super::super::mode::PaneRouteMode;

pub(super) fn route_local_dock_pane(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
    mode: PaneRouteMode,
) -> Option<PanePointerRoute> {
    let scene = &presentation.host_scene_data;
    if let Some(route) = route_document_dock_pane(&scene.document_dock, x, y, mode) {
        return Some(route);
    }
    if let Some(route) = route_side_dock_pane(&scene.left_dock, x, y, mode) {
        return Some(route);
    }
    if let Some(route) = route_side_dock_pane(&scene.right_dock, x, y, mode) {
        return Some(route);
    }
    if let Some(route) = route_bottom_dock_pane(&scene.bottom_dock, x, y, mode) {
        return Some(route);
    }
    None
}
