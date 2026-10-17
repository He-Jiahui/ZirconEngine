use super::*;
use crate::scene::viewport::SceneViewportChromeSettings;

#[test]
fn snap_snapshot_carries_each_of_the_eight_flag_combinations() {
    for mask in 0..8 {
        let mut settings = SceneViewportChromeSettings::default();
        settings.translate_snap_enabled = mask & 1 != 0;
        settings.rotate_snap_enabled = mask & 2 != 0;
        settings.scale_snap_enabled = mask & 4 != 0;
        let snapshot = viewport_snap_snapshot(&settings);
        let encoded = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(
            encoded,
            json!({"translationEnabled": mask & 1 != 0, "rotationEnabled": mask & 2 != 0, "scaleEnabled": mask & 4 != 0})
        );
        assert_eq!(
            serde_json::from_value::<ViewportSnapSnapshot>(encoded).unwrap(),
            snapshot
        );
    }
}

#[test]
fn snap_snapshot_rejects_absent_nonboolean_and_unknown_fields() {
    for invalid in [
        json!({}),
        json!({"translationEnabled": false, "rotationEnabled": true}),
        json!({"translationEnabled": false, "rotationEnabled": true, "scaleEnabled": null}),
        json!({"translationEnabled": false, "rotationEnabled": true, "scaleEnabled": 0}),
        json!({"translationEnabled": false, "rotationEnabled": true, "scaleEnabled": "false"}),
        json!({"translationEnabled": false, "rotationEnabled": true, "scaleEnabled": false, "gridVisible": true}),
    ] {
        assert!(serde_json::from_value::<ViewportSnapSnapshot>(invalid).is_err());
    }
}

#[test]
fn changing_one_axis_changes_the_canonical_snap_state_hash() {
    let mut settings = SceneViewportChromeSettings::default();
    let before = serde_json::to_value(viewport_snap_snapshot(&settings)).unwrap();
    settings.scale_snap_enabled = !settings.scale_snap_enabled;
    let after = serde_json::to_value(viewport_snap_snapshot(&settings)).unwrap();
    assert_ne!(
        canonical_hash(&before).unwrap(),
        canonical_hash(&after).unwrap()
    );
}
