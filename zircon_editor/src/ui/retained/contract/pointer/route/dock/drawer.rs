use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::visible_damage_frame;

pub(super) fn route_drawer_dock_damage_frame(
    presentation: &HostWindowPresentationData,
    surface_key: &str,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let frame = match surface_key {
        "left" => scene.left_dock.region_frame.clone(),
        "right" => scene.right_dock.region_frame.clone(),
        "bottom" => scene.bottom_dock.region_frame.clone(),
        _ => return None,
    };
    visible_damage_frame(frame)
}
