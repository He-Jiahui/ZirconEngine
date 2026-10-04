mod drawer;

use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use self::drawer::route_drawer_dock_damage_frame;
use super::super::union::visible_frame;

pub(super) fn route_activity_rail_damage_frame(
    presentation: &HostWindowPresentationData,
    side: &str,
    control_id: &str,
) -> Option<FrameRect> {
    let scene = &presentation.host_scene_data;
    let dock = match side {
        "left" => &scene.left_dock,
        "right" => &scene.right_dock,
        _ => return None,
    };
    let frame = if dock.rail_active_control_id.as_str() == control_id {
        presentation.host_layout.center_band_frame.clone()
    } else {
        dock.region_frame.clone()
    };
    visible_damage_frame(frame)
}

pub(super) fn route_document_dock_damage_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    visible_damage_frame(
        presentation
            .host_scene_data
            .document_dock
            .region_frame
            .clone(),
    )
}

pub(super) fn route_drawer_header_damage_frame(
    presentation: &HostWindowPresentationData,
    surface_key: &str,
) -> Option<FrameRect> {
    route_drawer_dock_damage_frame(presentation, surface_key)
}

pub(super) fn visible_damage_frame(frame: FrameRect) -> Option<FrameRect> {
    visible_frame(&frame).then_some(frame)
}

#[cfg(test)]
#[path = "tests/dock.rs"]
mod tests;
