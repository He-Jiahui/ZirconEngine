use super::super::super::data::HostWindowSceneData;
use super::super::geometry::floating_window_content_frame;
use super::pane::pane_route_hits_template;

pub(super) fn floating_windows_hit_template(
    scene: &HostWindowSceneData,
    id: &str,
    x: f32,
    y: f32,
) -> bool {
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
