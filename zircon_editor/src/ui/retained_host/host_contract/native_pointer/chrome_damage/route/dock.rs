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
mod tests {
    use super::route_activity_rail_damage_frame;
    use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

    #[test]
    fn inactive_activity_button_damages_only_its_dock() {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_layout.center_band_frame = frame(0.0, 64.0, 1200.0, 700.0);
        presentation.host_scene_data.left_dock.region_frame = frame(0.0, 64.0, 280.0, 700.0);
        presentation
            .host_scene_data
            .left_dock
            .rail_active_control_id = "active".into();

        assert_eq!(
            route_activity_rail_damage_frame(&presentation, "left", "next"),
            Some(frame(0.0, 64.0, 280.0, 700.0))
        );
    }

    #[test]
    fn active_activity_button_keeps_center_band_damage_for_collapse() {
        let mut presentation = HostWindowPresentationData::default();
        presentation.host_layout.center_band_frame = frame(0.0, 64.0, 1200.0, 700.0);
        presentation.host_scene_data.left_dock.region_frame = frame(0.0, 64.0, 280.0, 700.0);
        presentation
            .host_scene_data
            .left_dock
            .rail_active_control_id = "active".into();

        assert_eq!(
            route_activity_rail_damage_frame(&presentation, "left", "active"),
            Some(frame(0.0, 64.0, 1200.0, 700.0))
        );
    }

    fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
        FrameRect {
            x,
            y,
            width,
            height,
        }
    }
}
