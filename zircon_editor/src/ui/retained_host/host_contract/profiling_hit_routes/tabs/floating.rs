use super::super::super::data::HostWindowSceneData;
use super::super::geometry::{contains, translated};

pub(in crate::ui::retained_host::host_contract) fn floating_tab_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    surface: &str,
    x: f32,
    y: f32,
) -> bool {
    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        if window.window_id.as_str() != surface {
            continue;
        }
        let header = translated(&window.header_frame, window.frame.x, window.frame.y);
        for tab_row in 0..window.tab_frames.row_count() {
            let Some(tab) = window.tab_frames.row_data(tab_row) else {
                continue;
            };
            if tab.control_id.as_str() == id
                && contains(&translated(&tab.frame, header.x, header.y), x, y)
            {
                return true;
            }
        }
    }
    false
}
