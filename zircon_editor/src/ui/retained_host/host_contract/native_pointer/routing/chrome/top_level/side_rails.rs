use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;

use super::super::super::ChromePointerRoute;
use super::super::rails::route_activity_rail;

pub(super) fn route_side_activity_rails(
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<ChromePointerRoute> {
    let scene = &presentation.host_scene_data;
    route_activity_rail(
        &scene.left_dock.region_frame,
        true,
        scene.left_dock.rail_width_px,
        &scene.left_dock.rail_button_frames,
        x,
        y,
    )
    .or_else(|| {
        route_activity_rail(
            &scene.right_dock.region_frame,
            false,
            scene.right_dock.rail_width_px,
            &scene.right_dock.rail_button_frames,
            x,
            y,
        )
    })
}
