use super::dock_damage_route;
use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

#[test]
fn left_dock_damage_does_not_visit_unrelated_docks() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.region_frame = rect(0.0, 50.0, 240.0, 500.0);
    presentation.host_scene_data.document_dock.region_frame = rect(240.0, 50.0, 800.0, 500.0);
    presentation.host_scene_data.right_dock.region_frame = rect(1040.0, 50.0, 240.0, 500.0);
    presentation.host_scene_data.bottom_dock.region_frame = rect(0.0, 550.0, 1280.0, 170.0);

    let route = dock_damage_route(&presentation, Some(&rect(12.0, 72.0, 80.0, 32.0)));

    assert!(route.left);
    assert!(!route.document);
    assert!(!route.right);
    assert!(!route.bottom);
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}
