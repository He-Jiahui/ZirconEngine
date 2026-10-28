use super::*;
use crate::core::editing::engine::EditCommandError;
use crate::ui::workbench::state::EditorStateOperationError;
use zircon_runtime_interface::reflect::ReflectError;

#[test]
fn camera_projection_and_ortho_size_draft_commit_undo_redo() {
    let mut state = test_state();
    let (_, camera) = cube_and_camera(&state);
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    let names = [
        "projection_mode",
        "ortho_size",
        "fov_y_radians",
        "z_near",
        "z_far",
    ];
    let original = values(&state, camera, CAMERA, &names);
    let snapshot = state.snapshot();
    let inspector = snapshot.inspector.as_ref().unwrap();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    bridge
        .sync_scene_and_inspector(&snapshot.scene_entries, Some(inspector))
        .unwrap();

    for (name, draft) in [("projection_mode", "Orthographic"), ("ortho_size", "12.5")] {
        let field_id = format!("{CAMERA}.{name}");
        let field = inspector
            .native_fields
            .iter()
            .find(|field| field.field_id == field_id)
            .unwrap();
        assert!(field.editable);
        assert!(state.can_edit_dynamic_component_field(&field_id).unwrap());
        let control = row_for_field(&bridge, &field_id);
        assert!(bridge.control_bool_for_test(&control, "inspector_property_editable"));
        assert!(!bridge.control_bool_for_test(&control, "read_only"));
        assert_eq!(
            bridge
                .edit_inspector_component_property(
                    &control,
                    "Inspector/ComponentProperty04Edit",
                    draft,
                )
                .unwrap(),
            Some(true)
        );
        assert!(apply_inspector_draft_field(
            &mut state,
            "entity://selected",
            &field_id,
            draft.into(),
        )
        .unwrap());
    }
    assert_eq!(values(&state, camera, CAMERA, &names), original);
    let drafted = state.snapshot();
    let drafted_inspector = drafted.inspector.as_ref().unwrap();
    bridge
        .sync_scene_and_inspector(&drafted.scene_entries, Some(drafted_inspector))
        .unwrap();
    for (name, draft) in [("projection_mode", "Orthographic"), ("ortho_size", "12.5")] {
        assert_eq!(
            drafted_inspector
                .native_fields
                .iter()
                .find(|field| field.name == name)
                .unwrap()
                .value,
            draft
        );
    }

    assert!(apply_inspector_binding(
        &mut state,
        &binding(
            CAMERA,
            &[("projection_mode", "Orthographic"), ("ortho_size", "12.5")]
        ),
    )
    .unwrap());
    let mut edited = original.clone();
    edited[0] = ReflectedValue::Enum("Orthographic".to_string());
    edited[1] = ReflectedValue::Scalar(12.5);
    assert_eq!(values(&state, camera, CAMERA, &names), edited);
    assert_eq!(
        state
            .transactions()
            .history_status(state.scene_history_context().unwrap())
            .unwrap()
            .len,
        1
    );
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(values(&state, camera, CAMERA, &names), original);
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(values(&state, camera, CAMERA, &names), edited);

    assert!(apply_inspector_binding(
        &mut state,
        &binding(CAMERA, &[("projection_mode", "Perspective")]),
    )
    .unwrap());
    let mut perspective = edited.clone();
    perspective[0] = ReflectedValue::Enum("Perspective".to_string());
    assert_eq!(values(&state, camera, CAMERA, &names), perspective);
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(values(&state, camera, CAMERA, &names), edited);
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(values(&state, camera, CAMERA, &names), perspective);
}

#[test]
fn invalid_camera_projection_rejects_entire_ortho_batch_without_history() {
    let mut state = test_state();
    let (_, camera) = cube_and_camera(&state);
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    let names = ["projection_mode", "ortho_size"];
    let original = values(&state, camera, CAMERA, &names);
    // A valid scalar preceding an invalid enum must not be partially committed.
    let error = apply_inspector_binding(
        &mut state,
        &binding(
            CAMERA,
            &[("ortho_size", "12.5"), ("projection_mode", "Oblique")],
        ),
    )
    .unwrap_err();
    assert!(
        matches!(
            &error,
            EditorBindingDispatchError::State(EditorStateOperationError::EditCommand(
                EditCommandError::ReflectError { source }
            )) if matches!(
                source.downcast_ref::<ReflectError>(),
                Some(ReflectError::TypeMismatch { type_path, field_name, actual, .. })
                    if type_path == CAMERA && field_name == "projection_mode"
                        && actual == "Enum(Oblique)"
            )
        ),
        "{error:?}"
    );
    assert_eq!(values(&state, camera, CAMERA, &names), original);
    assert!(!state.snapshot().can_undo);
}

#[test]
fn static_camera_refuses_projection_and_ortho_size_including_direct_drafts() {
    let mut state = test_state();
    let (_, camera) = cube_and_camera(&state);
    state
        .world
        .expect_with_world_mut(|scene| scene.set_mobility(camera, Mobility::Static).unwrap());
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    let names = ["projection_mode", "ortho_size"];
    let original = values(&state, camera, CAMERA, &names);
    for (name, value) in [("projection_mode", "Orthographic"), ("ortho_size", "12.5")] {
        let error =
            apply_inspector_binding(&mut state, &binding(CAMERA, &[(name, value)])).unwrap_err();
        assert!(error.to_string().contains("static entity"), "{error:?}");
        assert_eq!(values(&state, camera, CAMERA, &names), original);
        assert!(!state.snapshot().can_undo);
    }
    state
        .update_dynamic_component_field(format!("{CAMERA}.projection_mode"), "Orthographic".into());
    state.update_dynamic_component_field(format!("{CAMERA}.ortho_size"), "12.5".into());
    let error = state.apply_inspector_changes().unwrap_err();
    assert!(error.to_string().contains("static entity"), "{error:?}");
    assert_eq!(values(&state, camera, CAMERA, &names), original);
    assert!(!state.snapshot().can_undo);
}
