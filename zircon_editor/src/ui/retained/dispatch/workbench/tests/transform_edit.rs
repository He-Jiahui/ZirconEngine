use super::*;
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn strip_axis_prefix_accepts_native_axis_labels() {
    assert_eq!(strip_axis_prefix("X 42.0", "X"), "42.0");
    assert_eq!(strip_axis_prefix("90 deg", "Y"), "90 deg");
}

#[test]
fn format_axis_row_value_keeps_reference_spacing() {
    assert_eq!(
        format_axis_row_value("12.0", "3.5", "-8.0"),
        "X 12.0   Y 3.5   Z -8.0"
    );
}

#[test]
fn transform_commit_emits_a_typed_finite_scalar() {
    let bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();

    let binding = bridge
        .transform_axis_commit_binding(POSITION_X, "Inspector/TransformPositionXCommit", "X 4.25")
        .unwrap()
        .expect("position commit should resolve");
    let EditorUiBindingPayload::InspectorFieldBatch { changes, .. } = binding.payload() else {
        panic!("position commit must dispatch an inspector field batch");
    };

    assert_eq!(changes[0].field_id, "transform.translation.x");
    assert_eq!(changes[0].value, UiBindingValue::Float(4.25));

    let binding = bridge
        .transform_axis_commit_binding(SCALE_Z, "Inspector/TransformScaleZCommit", "Z 2.5")
        .unwrap()
        .expect("scale commit should resolve");
    let EditorUiBindingPayload::InspectorFieldBatch { changes, .. } = binding.payload() else {
        panic!("scale commit must dispatch an inspector field batch");
    };

    assert_eq!(changes[0].field_id, "transform.scale.z");
    assert_eq!(changes[0].value, UiBindingValue::Float(2.5));
    assert_eq!(
        bridge
            .transform_axis_commit_binding(SCALE_Z, "Inspector/TransformScaleZCommit", "Z NaN",)
            .unwrap_err(),
        "Inspector transform Z value `NaN` must be a finite number"
    );
}

#[test]
fn rotation_retained_routes_edit_commit_and_disable_with_selection_lifecycle() {
    use crate::ui::binding_dispatch::apply_inspector_binding;
    use crate::ui::workbench::state::EditorState;
    use zircon_runtime::scene::{DefaultLevelManager, NodeKind};
    use zircon_runtime_interface::math::{EulerRot, Quat, UVec2};

    let mut state = EditorState::with_default_selection(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
    );
    state.mark_project_open();
    let cube = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .unwrap()
            .id
    });
    state
        .apply_intent(crate::core::editing::intent::EditorIntent::SelectNode(cube))
        .unwrap();
    let snapshot = state.snapshot();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    assert!(bridge
        .transform_axis_commit_binding(ROTATION_X, "Inspector/TransformRotationXCommit", "45")
        .unwrap()
        .is_none());
    bridge
        .sync_scene_and_inspector(&snapshot.scene_entries, snapshot.inspector.as_ref())
        .unwrap();

    for (control, axis) in [(ROTATION_X, "X"), (ROTATION_Y, "Y"), (ROTATION_Z, "Z")] {
        let edit_id = format!("Inspector/TransformRotation{axis}Edit");
        let commit_id = format!("Inspector/TransformRotation{axis}Commit");
        let projected = bridge
            .host_projection()
            .node_by_control_id(control)
            .unwrap();
        assert!(!projected.disabled);
        assert!(projected
            .routes
            .iter()
            .any(|route| route.binding_id == edit_id));
        assert!(projected
            .routes
            .iter()
            .any(|route| route.binding_id == commit_id));
        assert_eq!(
            bridge
                .edit_inspector_transform_axis(control, &edit_id, "45 deg")
                .unwrap(),
            Some(true)
        );
        assert_eq!(
            bridge.control_string(control, "value").as_deref(),
            Some("45 deg")
        );
        let commit = bridge
            .transform_axis_commit_binding(control, &commit_id, "45 deg")
            .unwrap()
            .unwrap();
        assert!(apply_inspector_binding(&mut state, &commit).unwrap());
        for invalid in ["NaN", "inf", "1e100", "", "45 deg junk"] {
            assert!(bridge
                .transform_axis_commit_binding(control, &commit_id, invalid)
                .is_err());
        }
        assert!(bridge
            .transform_axis_commit_binding(POSITION_X, &commit_id, "45")
            .unwrap()
            .is_none());
    }
    state.world.expect_with_world(|scene| {
        let actual = scene.local_transform(cube).unwrap().rotation;
        let angle = 45_f32.to_radians();
        assert!(
            actual
                .dot(Quat::from_euler(EulerRot::XYZ, angle, angle, angle))
                .abs()
                > 0.999999
        );
    });
    bridge
        .sync_scene_and_inspector(&snapshot.scene_entries, None)
        .unwrap();
    for control in [ROTATION_X, ROTATION_Y, ROTATION_Z] {
        assert!(bridge.control_bool(control, "disabled"));
    }
    assert!(bridge
        .transform_axis_commit_binding(ROTATION_X, "Inspector/TransformRotationXCommit", "45")
        .unwrap()
        .is_none());
    assert_eq!(
        bridge
            .edit_inspector_transform_axis(ROTATION_X, "Inspector/TransformRotationXEdit", "45")
            .unwrap(),
        Some(false)
    );
    let mut unavailable = snapshot.inspector.unwrap();
    unavailable.rotation_degrees = None;
    bridge
        .sync_scene_and_inspector(&snapshot.scene_entries, Some(&unavailable))
        .unwrap();
    assert!(bridge.control_bool(ROTATION_X, "disabled"));
    assert_eq!(
        bridge.control_string(ROTATION_X, "value").as_deref(),
        Some("—")
    );
}
