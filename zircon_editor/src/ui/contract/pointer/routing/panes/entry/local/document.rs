use crate::ui::retained_host::host_contract::data::{
    HostDocumentDockSurfaceData, HostWindowPresentationData,
};

use super::super::super::super::geometry::{contains, translated};
use super::super::super::super::PanePointerRoute;
use super::super::super::mode::PaneRouteMode;
use super::super::super::pane::pane_route_from_pane;

pub(super) fn route_document_dock_pane<'a>(
    presentation: &'a HostWindowPresentationData,
    dock: &'a HostDocumentDockSurfaceData,
    x: f32,
    y: f32,
    mode: PaneRouteMode,
    console_scroll_px: f32,
) -> Option<PanePointerRoute<'a>> {
    if let Some(viewport) = crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_level_viewport(
        presentation,
        dock,
    ) {
        if !contains(&viewport.clip, x, y) {
            return None;
        }
        let target = if dock.pane.kind.as_str() == "Scene" {
            super::super::super::super::PanePointerTarget::SceneViewport(dock.surface_key.as_str())
        } else {
            super::super::super::super::PanePointerTarget::GameViewport(dock.surface_key.as_str())
        };
        // Authored controls are dispatched through the workbench hit index first. The body
        // has no native toolbar/header inset and uses the same media frame as its GPU image.
        return Some(PanePointerRoute::new(target, &viewport.frame, x, y));
    }
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
        console_scroll_px,
    )
}
