use zircon_runtime::scene::{components::Mobility, NodeId, NodeKind};
use zircon_runtime_interface::reflect::ReflectedValue;
use zircon_runtime_interface::ui::{binding::UiBindingValue, layout::UiSize};

use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::InspectorFieldChange;
use crate::tests::editing::support::{cube_and_camera, test_state};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::binding_dispatch::{
    apply_inspector_binding, apply_inspector_draft_field, EditorBindingDispatchError,
};
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::workbench::snapshot::InspectorNativeFieldSnapshot;
use crate::ui::workbench::state::EditorState;

use super::support::read_reflected_field;

#[path = "native_fields/camera_projection.rs"]
mod camera_projection;
#[path = "native_fields/display_readonly.rs"]
mod display_readonly;
#[path = "native_fields/rigid_body.rs"]
mod rigid_body;

const CAMERA: &str = "zircon_runtime::scene::components::CameraComponent";
const POINT_LIGHT: &str = "zircon_runtime::scene::components::PointLight";
const MESH: &str = "zircon_runtime::scene::components::MeshRenderer";

fn binding(component: &str, fields: &[(&str, &str)]) -> EditorUiBinding {
    EditorUiBinding::new(
        "Inspector",
        "NativePropertyCommit",
        EditorUiEventKind::Submit,
        EditorUiBindingPayload::inspector_field_batch(
            "entity://selected",
            fields
                .iter()
                .map(|(name, value)| {
                    InspectorFieldChange::new(
                        format!("{component}.{name}"),
                        UiBindingValue::string(*value),
                    )
                })
                .collect::<Vec<_>>(),
        ),
    )
}

fn values(
    state: &EditorState,
    entity: NodeId,
    component: &str,
    fields: &[&str],
) -> Vec<ReflectedValue> {
    state.world.expect_with_world(|scene| {
        fields
            .iter()
            .map(|name| read_reflected_field(scene, entity, component, name))
            .collect()
    })
}

fn row_for_field(bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge, field_id: &str) -> String {
    bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            let metadata = node.template_metadata.as_ref()?;
            (metadata
                .attributes
                .get("inspector_property_field_id")
                .and_then(toml::Value::as_str)
                == Some(field_id))
            .then(|| metadata.control_id.clone())
            .flatten()
        })
        .unwrap_or_else(|| {
            let inventory = bridge
                .surface()
                .tree
                .nodes
                .values()
                .filter_map(|node| {
                    let metadata = node.template_metadata.as_ref()?;
                    let control = metadata.control_id.as_deref()?;
                    (control.contains("Property") || control.contains("InspectorMesh")).then(|| {
                        (
                            control.to_owned(),
                            metadata
                                .attributes
                                .get("inspector_property_field_id")
                                .cloned(),
                            node.visibility,
                            node.layout_cache.frame,
                        )
                    })
                })
                .collect::<Vec<_>>();
            panic!("actual field must materialize: {field_id}; surface rows: {inventory:?}")
        })
}

#[test]
fn camera_fov_details_draft_commit_undo_redo_uses_reflected_commands() {
    let mut state = test_state();
    let (_, camera) = cube_and_camera(&state);
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    let names = ["fov_y_radians", "z_near", "z_far"];
    let original = values(&state, camera, CAMERA, &names);
    let snapshot = state.snapshot();
    let inspector = snapshot.inspector.as_ref().unwrap();
    for name in names {
        assert!(
            inspector
                .native_fields
                .iter()
                .find(|field| field.field_id == format!("{CAMERA}.{name}"))
                .unwrap()
                .editable
        );
    }
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    bridge
        .sync_scene_and_inspector(&snapshot.scene_entries, Some(inspector))
        .unwrap();
    let fov_id = format!("{CAMERA}.fov_y_radians");
    let fov_control = row_for_field(&bridge, &fov_id);
    assert!(bridge.control_bool_for_test(&fov_control, "inspector_property_editable"));
    assert!(!bridge.control_bool_for_test(&fov_control, "read_only"));
    assert_eq!(
        bridge
            .edit_inspector_component_property(
                &fov_control,
                "Inspector/ComponentProperty04Edit",
                "0.9",
            )
            .unwrap(),
        Some(true)
    );
    assert!(
        apply_inspector_draft_field(&mut state, "entity://selected", &fov_id, "0.9".into())
            .unwrap()
    );
    assert_eq!(values(&state, camera, CAMERA, &names), original);
    let drafted = state.snapshot();
    assert_eq!(
        drafted
            .inspector
            .unwrap()
            .native_fields
            .iter()
            .find(|field| field.field_id == fov_id)
            .unwrap()
            .value,
        "0.9"
    );

    assert!(apply_inspector_binding(
        &mut state,
        &binding(
            CAMERA,
            &[
                ("fov_y_radians", "0.9"),
                ("z_near", "0.25"),
                ("z_far", "500")
            ]
        ),
    )
    .unwrap());
    let edited = vec![
        ReflectedValue::Scalar(0.9),
        ReflectedValue::Scalar(0.25),
        ReflectedValue::Scalar(500.0),
    ];
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
}

#[test]
fn light_colour_intensity_range_and_shadow_details_commit_undo_redo() {
    let mut state = test_state();
    let light = state
        .world
        .expect_with_world_mut(|scene| scene.spawn_node(NodeKind::PointLight).unwrap());
    state.apply_intent(EditorIntent::SelectNode(light)).unwrap();
    let names = ["color", "intensity", "range", "casts_shadow"];
    let original = values(&state, light, POINT_LIGHT, &names);
    let snapshot = state.snapshot();
    for name in names {
        assert!(
            snapshot
                .inspector
                .as_ref()
                .unwrap()
                .native_fields
                .iter()
                .find(|field| field.name == name)
                .unwrap()
                .editable
        );
    }
    assert!(apply_inspector_binding(
        &mut state,
        &binding(
            POINT_LIGHT,
            &[
                ("color", "[0.25, 0.5, 0.75]"),
                ("intensity", "6"),
                ("range", "15"),
                ("casts_shadow", "true")
            ],
        ),
    )
    .unwrap());
    let edited = vec![
        ReflectedValue::Vec3([0.25, 0.5, 0.75]),
        ReflectedValue::Scalar(6.0),
        ReflectedValue::Scalar(15.0),
        ReflectedValue::Bool(true),
    ];
    assert_eq!(values(&state, light, POINT_LIGHT, &names), edited);
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(values(&state, light, POINT_LIGHT, &names), original);
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(values(&state, light, POINT_LIGHT, &names), edited);
}

#[test]
fn native_projection_honours_write_capability_and_keeps_resources_read_only() {
    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    let mut fields = state.world.expect_with_world(|scene| {
        scene
            .inspection_fields_artifact(camera)
            .unwrap()
            .fields()
            .to_vec()
    });
    fields.iter_mut().for_each(|field| field.writable = false);
    assert!(InspectorNativeFieldSnapshot::project(&fields)
        .iter()
        .all(|field| !field.editable));
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    let snapshot = state.snapshot();
    let inspector = snapshot.inspector.as_ref().unwrap();
    assert!(inspector
        .native_fields
        .iter()
        .any(|field| field.resource_id.is_some()));
    for field in inspector
        .native_fields
        .iter()
        .filter(|field| field.resource_id.is_some())
    {
        assert!(!field.editable);
        assert!(!state
            .can_edit_dynamic_component_field(&field.field_id)
            .unwrap());
    }
    let mut fields = state.world.expect_with_world(|scene| {
        scene
            .inspection_fields_artifact(cube)
            .unwrap()
            .fields()
            .to_vec()
    });
    fields.iter_mut().for_each(|field| field.writable = true);
    assert!(InspectorNativeFieldSnapshot::project(&fields)
        .iter()
        .filter(|field| field.resource_id.is_some())
        .all(|field| !field.editable));
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    bridge
        .sync_scene_and_inspector(&snapshot.scene_entries, Some(inspector))
        .unwrap();
    let resource = inspector
        .native_fields
        .iter()
        .find(|field| field.resource_id.is_some())
        .unwrap();
    let resource_control = row_for_field(&bridge, &resource.field_id);
    assert!(!bridge.control_bool_for_test(&resource_control, "inspector_property_editable"));
    assert_eq!(
        bridge
            .edit_inspector_component_property(
                &resource_control,
                "Inspector/ComponentProperty04Edit",
                "replacement"
            )
            .unwrap(),
        Some(false)
    );
    let original = values(&state, cube, MESH, &["material"]);
    assert!(matches!(
        apply_inspector_binding(&mut state, &binding(MESH, &[("material", "replacement")])).unwrap_err(),
        EditorBindingDispatchError::UnsupportedInspectorField(field) if field == format!("{MESH}.material")
    ));
    // Direct draft insertion must not bypass the resource policy either.
    state.update_dynamic_component_field(format!("{MESH}.material"), "replacement".into());
    assert!(state.apply_inspector_changes().is_err());
    assert_eq!(values(&state, cube, MESH, &["material"]), original);
    assert!(!state.snapshot().can_undo);
}

#[test]
fn native_camera_static_and_invalid_light_edits_leave_values_and_history_unchanged() {
    let mut state = test_state();
    let (_, camera) = cube_and_camera(&state);
    state
        .world
        .expect_with_world_mut(|scene| scene.set_mobility(camera, Mobility::Static).unwrap());
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    let original = values(&state, camera, CAMERA, &["fov_y_radians"]);
    let error = apply_inspector_binding(&mut state, &binding(CAMERA, &[("fov_y_radians", "0.9")]))
        .unwrap_err();
    assert!(error.to_string().contains("static entity"), "{error:?}");
    assert_eq!(values(&state, camera, CAMERA, &["fov_y_radians"]), original);
    assert!(!state.snapshot().can_undo);

    let light = state
        .world
        .expect_with_world_mut(|scene| scene.spawn_node(NodeKind::PointLight).unwrap());
    state.apply_intent(EditorIntent::SelectNode(light)).unwrap();
    let original = values(&state, light, POINT_LIGHT, &["color", "intensity"]);
    for invalid in ["1, bad, 0", "NaN, 0, 1"] {
        assert!(apply_inspector_binding(
            &mut state,
            &binding(POINT_LIGHT, &[("color", invalid), ("intensity", "10")])
        )
        .is_err());
        assert_eq!(
            values(&state, light, POINT_LIGHT, &["color", "intensity"]),
            original
        );
        assert!(!state.snapshot().can_undo);
    }
}

#[test]
fn native_camera_play_edits_use_play_history_and_keep_resources_protected() {
    use crate::core::gateway::InProcessGateway;
    use crate::core::play::PlayInstanceId;
    use std::sync::Arc;
    use zircon_runtime::scene::DefaultLevelManager;

    let mut state = test_state();
    let (cube, camera) = cube_and_camera(&state);
    state
        .apply_intent(EditorIntent::SelectNode(camera))
        .unwrap();
    let authored = values(&state, camera, CAMERA, &["fov_y_radians"]);
    assert!(state.enter_play_mode().unwrap());
    let instance = PlayInstanceId::for_test(103);
    let play_level = DefaultLevelManager::default().create_default_level();
    state
        .context
        .play_gateway_handle()
        .replace_for_play(
            Arc::new(InProcessGateway::for_authoring_level(play_level.clone())),
            Some(instance.raw()),
        )
        .unwrap();
    assert!(state.activate_play_selection_domain(instance));
    assert!(
        apply_inspector_binding(&mut state, &binding(CAMERA, &[("fov_y_radians", "0.9")])).unwrap()
    );
    let play_fov = || {
        play_level.with_world(|scene| read_reflected_field(scene, camera, CAMERA, "fov_y_radians"))
    };
    assert_eq!(play_fov(), ReflectedValue::Scalar(0.9));
    assert_eq!(values(&state, camera, CAMERA, &["fov_y_radians"]), authored);
    assert!(state.apply_intent(EditorIntent::Undo).unwrap());
    assert_eq!(play_fov(), authored[0]);
    assert!(state.apply_intent(EditorIntent::Redo).unwrap());
    assert_eq!(play_fov(), ReflectedValue::Scalar(0.9));

    state
        .select_node_in_world(crate::core::play::WorldDomain::Play(instance), cube)
        .unwrap();
    let material =
        play_level.with_world(|scene| read_reflected_field(scene, cube, MESH, "material"));
    assert!(
        apply_inspector_binding(&mut state, &binding(MESH, &[("material", "replacement")]))
            .is_err()
    );
    play_level.with_world(|scene| {
        assert_eq!(
            read_reflected_field(scene, cube, MESH, "material"),
            material
        )
    });
    assert_eq!(values(&state, camera, CAMERA, &["fov_y_radians"]), authored);
}
