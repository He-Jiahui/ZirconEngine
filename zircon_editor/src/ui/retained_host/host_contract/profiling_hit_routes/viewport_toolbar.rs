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
    pane_route_hits_viewport_toolbar(
        id,
        x,
        y,
        scene.document_dock.surface_key.as_str(),
        &scene.document_dock.pane,
        &translated(
            &scene.document_dock.content_frame,
            scene.document_dock.region_frame.x,
            scene.document_dock.region_frame.y,
        ),
    ) || pane_route_hits_viewport_toolbar(
        id,
        x,
        y,
        scene.left_dock.surface_key.as_str(),
        &scene.left_dock.pane,
        &side_dock_content_frame(&scene.left_dock),
    ) || pane_route_hits_viewport_toolbar(
        id,
        x,
        y,
        scene.right_dock.surface_key.as_str(),
        &scene.right_dock.pane,
        &side_dock_content_frame(&scene.right_dock),
    ) || pane_route_hits_viewport_toolbar(
        id,
        x,
        y,
        scene.bottom_dock.surface_key.as_str(),
        &scene.bottom_dock.pane,
        &translated(
            &scene.bottom_dock.content_frame,
            scene.bottom_dock.region_frame.x,
            scene.bottom_dock.region_frame.y,
        ),
    ) || floating_windows_hit_toolbar(scene, id, x, y)
}
