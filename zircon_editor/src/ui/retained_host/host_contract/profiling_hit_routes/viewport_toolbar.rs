mod floating;
mod pane;
mod route_check;

use self::floating::floating_windows_hit_toolbar;
use self::pane::pane_route_hits_viewport_toolbar;
use super::super::data::HostWindowSceneData;
use super::geometry::{side_dock_content_frame, translated};

pub(in crate::ui::retained_host::host_contract) fn viewport_toolbar_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    x: f32,
    y: f32,
) -> bool {
    scene.document_surfaces().iter().any(|document| {
        let surface = if scene.document_leaves.is_empty() {
            "document"
        } else {
            document.surface_key.as_str()
        };
        pane_route_hits_viewport_toolbar(
            id,
            x,
            y,
            surface,
            &document.pane,
            &translated(
                &document.content_frame,
                document.region_frame.x,
                document.region_frame.y,
            ),
        )
    }) || pane_route_hits_viewport_toolbar(
        id,
        x,
        y,
        "left",
        &scene.left_dock.pane,
        &side_dock_content_frame(&scene.left_dock),
    ) || pane_route_hits_viewport_toolbar(
        id,
        x,
        y,
        "right",
        &scene.right_dock.pane,
        &side_dock_content_frame(&scene.right_dock),
    ) || pane_route_hits_viewport_toolbar(
        id,
        x,
        y,
        "bottom",
        &scene.bottom_dock.pane,
        &translated(
            &scene.bottom_dock.content_frame,
            scene.bottom_dock.region_frame.x,
            scene.bottom_dock.region_frame.y,
        ),
    ) || floating_windows_hit_toolbar(scene, id, x, y)
}
