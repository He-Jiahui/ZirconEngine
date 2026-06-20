use zircon_runtime::ui::surface::hit_test_surface_frame;
use zircon_runtime_interface::ui::layout::UiPoint;

use super::super::data::{FrameRect, HostWindowSceneData, PaneData};
use super::geometry::{
    contains, floating_window_content_frame, side_dock_content_frame, translated,
};

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

fn pane_route_hits_template(
    id: &str,
    x: f32,
    y: f32,
    surface: &str,
    pane: &PaneData,
    content: &FrameRect,
) -> bool {
    let expected_prefix = format!("template.{surface}.");
    if !id.starts_with(&expected_prefix) || !contains(content, x, y) {
        return false;
    }
    let mut body = content.clone();
    if matches!(pane.kind.as_str(), "Scene" | "Game") && pane.show_toolbar {
        let toolbar_height = 28.0_f32.min(content.height);
        body.y += toolbar_height;
        body.height = (body.height - toolbar_height).max(0.0);
    }
    let Some(surface_frame) = pane.body_surface_frame.as_ref() else {
        return false;
    };
    let point = UiPoint::new(x - body.x, y - body.y);
    let Some(node_id) = hit_test_surface_frame(surface_frame, point).top_hit else {
        return false;
    };
    let Some(node) = surface_frame.arranged_tree.get(node_id) else {
        return false;
    };
    let Some(control_id) = node.control_id.as_deref() else {
        return false;
    };
    format!("template.{surface}.{control_id}") == id
}

fn floating_windows_hit_template(scene: &HostWindowSceneData, id: &str, x: f32, y: f32) -> bool {
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        if pane_route_hits_template(
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
