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
