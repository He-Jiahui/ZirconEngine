use super::{control_route_for_id, route_for_control};
use crate::ui::retained_host::viewport_toolbar_pointer::ViewportToolbarPointerRoute;

#[test]
fn every_legacy_control_and_alias_has_one_descriptor() {
    for control_id in [
        "mode.select",
        "mode.move",
        "mode.rotate",
        "mode.scale",
        "space.local",
        "transform.local",
        "space.global",
        "transform.global",
        "pivot.cycle",
        "projection.perspective",
        "projection.orthographic",
        "align.pos_x",
        "align.neg_x",
        "align.pos_y",
        "align.neg_y",
        "align.pos_z",
        "align.neg_z",
        "display.cycle",
        "grid.cycle",
        "snap.translate",
        "translate_snap.cycle",
        "snap.rotate",
        "rotate_snap.cycle",
        "snap.scale",
        "scale_snap.cycle",
        "toggle.lighting",
        "preview_lighting.toggle",
        "toggle.skybox",
        "preview_skybox.toggle",
        "toggle.gizmos",
        "gizmos.toggle",
        "frame.selection",
        "frame_selection",
        "EnterPlayMode",
        "ExitPlayMode",
    ] {
        assert!(
            control_route_for_id(control_id).is_some(),
            "missing descriptor for {control_id}"
        );
    }
    assert!(control_route_for_id("unknown").is_none());
    assert!(control_route_for_id("mode.custom:").is_none());
}

#[test]
fn custom_scene_mode_is_owned_only_when_the_route_is_materialized() {
    assert_eq!(
        route_for_control("scene.main", "mode.custom:terrain").unwrap(),
        ViewportToolbarPointerRoute::ActivateSceneMode {
            surface_key: "scene.main".to_string(),
            mode: "Custom:terrain".to_string(),
        }
    );
}
