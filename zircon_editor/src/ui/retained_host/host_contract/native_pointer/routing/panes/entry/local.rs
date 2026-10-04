mod bottom;
mod document;
mod side;

use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;

use self::bottom::route_bottom_dock_pane;
use self::document::route_document_dock_pane;
use self::side::route_side_dock_pane;
use super::super::super::PanePointerRoute;
use super::super::mode::PaneRouteMode;

pub(super) fn route_local_dock_pane<'a>(
    presentation: &'a HostWindowPresentationData,
    x: f32,
    y: f32,
    mode: PaneRouteMode,
    console_scroll_px: f32,
) -> Option<PanePointerRoute<'a>> {
    let scene = &presentation.host_scene_data;
    if let Some(hierarchy) =
        crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_hierarchy(
            presentation,
        )
    {
        if super::super::super::geometry::contains(&hierarchy.viewport, x, y) {
            let mut route = PanePointerRoute::new(
                super::super::super::PanePointerTarget::Hierarchy,
                &hierarchy.viewport,
                x,
                y,
            );
            route.hierarchy_row_metrics = Some(hierarchy.metrics);
            return Some(route);
        }
    }
    for document in scene.document_surfaces() {
        if let Some(route) = route_document_dock_pane(document, x, y, mode, console_scroll_px) {
            return Some(route);
        }
    }
    if !crate::ui::retained_host::host_contract::componentized_workbench_regions::owns_ordinary_panes(presentation) {
    if let Some(route) = route_side_dock_pane(&scene.left_dock, x, y, mode, console_scroll_px) {
        return Some(route);
    }
    if let Some(route) = route_side_dock_pane(&scene.right_dock, x, y, mode, console_scroll_px) {
        return Some(route);
    }
    }
    if let Some(route) = route_bottom_dock_pane(&scene.bottom_dock, x, y, mode, console_scroll_px) {
        return Some(route);
    }
    None
}
