mod floating;
mod pane;
mod route_check;

use self::floating::floating_windows_hit_template;
use self::pane::pane_route_hits_template;
use super::super::data::HostWindowSceneData;
use super::geometry::{side_dock_content_frame, translated};

pub(in crate::ui::retained_host::host_contract) fn template_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    x: f32,
    y: f32,
) -> bool {
    pane_route_hits_template(
        id,
        x,
        y,
        "document",
        &scene.document_dock.pane,
        &translated(
            &scene.document_dock.content_frame,
            scene.document_dock.region_frame.x,
            scene.document_dock.region_frame.y,
        ),
    ) || pane_route_hits_template(
        id,
        x,
        y,
        "left",
        &scene.left_dock.pane,
        &side_dock_content_frame(&scene.left_dock),
    ) || pane_route_hits_template(
        id,
        x,
        y,
        "right",
        &scene.right_dock.pane,
        &side_dock_content_frame(&scene.right_dock),
    ) || pane_route_hits_template(
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
    ) || floating_windows_hit_template(scene, id, x, y)
}
