use super::super::data::HostWindowSceneData;
use super::geometry::{contains, translated};

pub(in crate::ui::retained_host::host_contract) fn resize_splitter_route_hit(
    scene: &HostWindowSceneData,
    surface: &str,
    x: f32,
    y: f32,
) -> bool {
    match surface {
        "left" => contains(&scene.resize_layer.left_splitter_frame, x, y),
        "right" => contains(&scene.resize_layer.right_splitter_frame, x, y),
        "bottom" => contains(&scene.resize_layer.bottom_splitter_frame, x, y),
        _ => false,
    }
}

pub(in crate::ui::retained_host::host_contract) fn activity_rail_route_hit(
    scene: &HostWindowSceneData,
    id: &str,
    surface: &str,
    x: f32,
    y: f32,
) -> bool {
    let dock = match surface {
        "left" => &scene.left_dock,
        "right" => &scene.right_dock,
        _ => return false,
    };
    if dock.rail_width_px <= 0.0 || !contains(&dock.region_frame, x, y) {
        return false;
    }
    let rail_x = if dock.rail_before_panel {
        dock.region_frame.x
    } else {
        dock.region_frame.x + (dock.region_frame.width - dock.rail_width_px).max(0.0)
    };
    let rail = super::super::data::FrameRect {
        x: rail_x,
        y: dock.region_frame.y,
        width: dock.rail_width_px.min(dock.region_frame.width.max(0.0)),
        height: dock.region_frame.height,
    };
    for row in 0..dock.rail_button_frames.row_count() {
        let Some(button) = dock.rail_button_frames.row_data(row) else {
            continue;
        };
        let expected_id = format!("activity_rail.{surface}.{}", button.control_id);
        if expected_id == id && contains(&translated(&button.frame, rail.x, rail.y), x, y) {
            return true;
        }
    }
    false
}
