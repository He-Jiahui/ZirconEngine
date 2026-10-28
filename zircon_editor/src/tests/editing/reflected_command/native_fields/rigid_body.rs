use super::*;
use zircon_runtime::core::framework::scene::physics::PhysicsMassProperties;
use zircon_runtime::scene::components::RigidBodyComponent;

const RIGID_BODY: &str = "zircon_runtime::scene::components::RigidBodyComponent";

fn fixture() -> (EditorState, NodeId, RigidBodyComponent) {
    let state = test_state();
    let (entity, _) = cube_and_camera(&state);
    let original = RigidBodyComponent {
        mass: 2.0,
        mass_properties: PhysicsMassProperties::Explicit {
            inertia_tensor: Some([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 4.0]]),
        },
        ..Default::default()
    };
    state.world.expect_with_world_mut(|scene| {
        scene
            .set_rigid_body(entity, Some(original.clone()))
            .unwrap()
    });
    (state, entity, original)
}

fn body(state: &EditorState, entity: NodeId) -> RigidBodyComponent {
    state
        .world
        .expect_with_world(|scene| scene.rigid_body(entity).unwrap().clone())
}

#[test]
fn rigid_body_displays_existing_safe_editors_and_roundtrips_details_with_exact_undo() {
    let (mut state, entity, original) = fixture();
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .unwrap();
    let edits = [
        ("mass", "3.5"),
        ("linear_damping", "0.25"),
        ("angular_damping", "0.5"),
        ("gravity_scale", "0.75"),
        ("ccd_mode", "LinearCast"),
        ("sleep_policy", "Never"),
    ];
    let snapshot = state.snapshot();
    let inspector = snapshot.inspector.as_ref().unwrap();
    let rows = inspector
        .native_fields
        .iter()
        .filter(|field| field.component_type_path == RIGID_BODY)
        .collect::<Vec<_>>();
    assert_eq!(rows.len(), 13);
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    bridge
        .sync_scene_and_inspector(&snapshot.scene_entries, Some(inspector))
        .unwrap();
    for (name, draft) in edits {
        let field_id = format!("{RIGID_BODY}.{name}");
        let field = rows.iter().find(|field| field.name == name).unwrap();
        assert_eq!(field.component_label, "Rigid Body");
        assert!(field.editable);
        assert!(state.can_edit_dynamic_component_field(&field_id).unwrap());
        bridge
            .edit_inspector_filter(
                "WorkbenchInspectorFilter",
                "Workbench/InspectorSearchEdit",
                &field_id,
            )
            .unwrap();
        let control = row_for_field(&bridge, &field_id);
        assert!(bridge.control_bool_for_test(&control, "inspector_property_editable"));
        assert_eq!(
            bridge
                .edit_inspector_component_property(
                    &control,
                    "Inspector/ComponentProperty04Edit",
                    draft
                )
                .unwrap(),
            Some(true)
        );
        assert!(apply_inspector_draft_field(
            &mut state,
            "entity://selected",
            &field_id,
            draft.into()
        )
        .unwrap());
    }
    assert_eq!(body(&state, entity), original);
    let drafted = state.snapshot();
    for (name, draft) in edits {
        assert_eq!(
            drafted
                .inspector
                .as_ref()
                .unwrap()
                .native_fields
                .iter()
                .find(|field| field.field_id == format!("{RIGID_BODY}.{name}"))
                .unwrap()
                .value,
            draft
        );
    }
    assert!(apply_inspector_binding(&mut state, &binding(RIGID_BODY, &edits)).unwrap());
    let mut edited = original.clone();
    edited.mass = 3.5;
    edited.linear_damping = 0.25;
    edited.angular_damping = 0.5;
    edited.gravity_scale = 0.75;
    edited.ccd_mode = zircon_runtime::core::framework::scene::physics::PhysicsCcdMode::LinearCast;
    edited.sleep_policy =
        zircon_runtime::core::framework::scene::physics::PhysicsSleepPolicy::Never;
    assert_eq!(body(&state, entity), edited);
    assert_eq!(
        state
            .transactions()
            .history_status(state.scene_history_context().unwrap())
            .unwrap()
            .len,
        1
    );
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(body(&state, entity), original);
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(body(&state, entity), edited);
}

#[test]
fn rigid_body_keeps_lossy_virtuals_and_runtime_state_readonly_even_for_direct_drafts() {
    let (mut state, entity, original) = fixture();
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .unwrap();
    let snapshot = state.snapshot();
    for (name, draft) in [
        ("body_type", "Kinematic"),
        ("linear_velocity", "1, 2, 3"),
        ("angular_velocity", "1, 2, 3"),
        ("lock_translation", "true"),
        ("lock_rotation", "true"),
        ("mass_properties_mode", "AutoFromShape"),
        ("mass_density", "2"),
    ] {
        let field_id = format!("{RIGID_BODY}.{name}");
        assert!(
            !snapshot
                .inspector
                .as_ref()
                .unwrap()
                .native_fields
                .iter()
                .find(|field| field.field_id == field_id)
                .unwrap()
                .editable
        );
        assert!(!state.can_edit_dynamic_component_field(&field_id).unwrap());
        assert!(
            apply_inspector_binding(&mut state, &binding(RIGID_BODY, &[(name, draft)])).is_err()
        );
        state.update_dynamic_component_field(field_id, draft.into());
        assert!(state.apply_inspector_changes().is_err());
        state.clear_inspector_draft_edits();
        assert_eq!(body(&state, entity), original);
        assert!(!state.snapshot().can_undo);
    }
}

#[test]
fn static_rigid_body_rejects_safe_fields_including_direct_drafts_without_history() {
    let (mut state, entity, original) = fixture();
    state
        .world
        .expect_with_world_mut(|scene| scene.set_mobility(entity, Mobility::Static).unwrap());
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .unwrap();
    for (name, draft) in [("mass", "4"), ("ccd_mode", "LinearCast")] {
        let error = apply_inspector_binding(&mut state, &binding(RIGID_BODY, &[(name, draft)]))
            .unwrap_err();
        assert!(error.to_string().contains("static entity"), "{error:?}");
        state.update_dynamic_component_field(format!("{RIGID_BODY}.{name}"), draft.into());
        let error = state.apply_inspector_changes().unwrap_err();
        assert!(error.to_string().contains("static entity"), "{error:?}");
        state.clear_inspector_draft_edits();
        assert_eq!(body(&state, entity), original);
        assert!(!state.snapshot().can_undo);
        assert!(!state.snapshot().can_redo);
    }
}
