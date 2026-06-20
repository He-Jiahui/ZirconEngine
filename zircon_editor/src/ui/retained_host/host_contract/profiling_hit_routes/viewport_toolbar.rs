use super::super::data::{FrameRect, HostWindowSceneData, PaneData};
use super::super::surface_hit_test;
use super::geometry::{
    contains, floating_window_content_frame, side_dock_content_frame, translated,
};

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

fn pane_route_hits_viewport_toolbar(
    id: &str,
    x: f32,
    y: f32,
    surface_key: &str,
    pane: &PaneData,
    content: &FrameRect,
) -> bool {
    let expected_prefix = format!("viewport_toolbar_control.{surface_key}.");
    if !id.starts_with(&expected_prefix)
        || !matches!(pane.kind.as_str(), "Scene" | "Game")
        || !pane.show_toolbar
        || !contains(content, x, y)
    {
        return false;
    }
    let toolbar_height = 28.0_f32.min(content.height);
    let toolbar = FrameRect {
        x: content.x,
        y: content.y,
        width: content.width,
        height: toolbar_height,
    };
    surface_hit_test::hit_test_viewport_toolbar(surface_key, &pane.viewport, &toolbar, x, y)
        .is_some_and(|hit| {
            format!("viewport_toolbar_control.{surface_key}.{}", hit.control_id) == id
        })
}

fn floating_windows_hit_toolbar(scene: &HostWindowSceneData, id: &str, x: f32, y: f32) -> bool {
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        if pane_route_hits_viewport_toolbar(
            id,
            x,
            y,
            window.window_id.as_str(),
            &window.active_pane,
            &floating_window_content_frame(&window.frame, &window.header_frame),
        ) {
            return true;
        }
    }
    false
}
