use super::*;
use zircon_runtime::core::resource::{ResourceHandle, ResourceId};
use zircon_runtime::scene::components::{
    AnimationPlayerComponent, ColliderComponent, ColliderShape, JointComponent, JointKind,
    PostProcessSettingsComponent, PostProcessVolumeComponent, Sprite2dComponent,
};
use zircon_runtime_interface::math::{Vec2, Vec3, Vec4};

fn assert_readonly_details(
    state: &mut EditorState,
    entity: NodeId,
    component: &str,
    label: &str,
    expected: &[(&str, ReflectedValue)],
) {
    state
        .apply_intent(EditorIntent::SelectNode(entity))
        .unwrap();
    let snapshot = state.snapshot();
    let inspector = snapshot.inspector.as_ref().unwrap();
    assert!(inspector.plugin_components.is_empty());
    let projected = inspector
        .native_fields
        .iter()
        .filter(|field| field.component_type_path == component)
        .collect::<Vec<_>>();
    assert_eq!(projected.len(), expected.len(), "{component}");
    let names = expected.iter().map(|(name, _)| *name).collect::<Vec<_>>();
    let original = expected
        .iter()
        .map(|(_, value)| value.clone())
        .collect::<Vec<_>>();
    assert_eq!(values(state, entity, component, &names), original);
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
    bridge
        .sync_scene_and_inspector(&snapshot.scene_entries, Some(inspector))
        .unwrap();
    for (name, value) in expected {
        let field_id = format!("{component}.{name}");
        let field = projected.iter().find(|field| field.name == *name).unwrap();
        assert_eq!(field.component_label, label);
        assert!(!field.editable, "{field_id}");
        assert!(!state.can_edit_dynamic_component_field(&field_id).unwrap());
        let displayed = match value {
            ReflectedValue::Bool(value) => value.to_string(),
            ReflectedValue::Unsigned(value) => value.to_string(),
            ReflectedValue::Integer(value) => value.to_string(),
            ReflectedValue::Scalar(value) => value.to_string(),
            ReflectedValue::Enum(value) => value.clone(),
            // The fixture has no asset catalog; the identity is checked separately below.
            ReflectedValue::Resource(_) => "Unavailable".into(),
            ReflectedValue::Vec2(value) => format!("{}, {}", value[0], value[1]),
            ReflectedValue::Vec3(value) => format!("{}, {}, {}", value[0], value[1], value[2]),
            ReflectedValue::Vec4(value) => {
                format!("{}, {}, {}, {}", value[0], value[1], value[2], value[3])
            }
            ReflectedValue::Entity(Some(value)) => value.to_string(),
            _ => panic!("missing display expectation for {field_id}"),
        };
        assert_eq!(field.value, displayed, "{field_id}");
        if let ReflectedValue::Resource(id) = value {
            assert_eq!(field.resource_id.as_ref(), Some(id));
        }
        // Search materializes the actual virtual row without increasing its viewport budget.
        bridge
            .edit_inspector_filter(
                "WorkbenchInspectorFilter",
                "Workbench/InspectorSearchEdit",
                &field_id,
            )
            .unwrap();
        let control = row_for_field(&bridge, &field_id);
        assert!(!bridge.control_bool_for_test(&control, "inspector_property_editable"));
        assert!(bridge.control_bool_for_test(&control, "read_only"));
        assert_eq!(
            bridge
                .edit_inspector_component_property(
                    &control,
                    "Inspector/ComponentProperty04Edit",
                    "7"
                )
                .unwrap(),
            Some(false)
        );
        assert!(
            apply_inspector_draft_field(state, "entity://selected", &field_id, "7".into()).is_err()
        );
        assert!(matches!(
            apply_inspector_binding(state, &binding(component, &[(name, "7")])).unwrap_err(),
            EditorBindingDispatchError::UnsupportedInspectorField(id) if id == field_id
        ));
    }
    // Neither direct draft insertion nor changing mobility bypasses the reflected capability.
    for mobility in [Mobility::Dynamic, Mobility::Static] {
        state
            .world
            .expect_with_world_mut(|scene| scene.set_mobility(entity, mobility).unwrap());
        state.update_dynamic_component_field(format!("{component}.{}", names[0]), "7".into());
        assert!(state.apply_inspector_changes().is_err());
        state.clear_inspector_draft_edits();
        assert!(apply_inspector_binding(state, &binding(component, &[(names[0], "7")])).is_err());
        assert_eq!(values(state, entity, component, &names), original);
        let snapshot = state.snapshot();
        assert!(!snapshot.can_undo);
        assert!(!snapshot.can_redo);
        assert!(snapshot
            .inspector
            .unwrap()
            .native_fields
            .iter()
            .filter(|field| field.component_type_path == component)
            .all(|field| !field.editable));
    }
}

#[test]
fn collider_displays_configuration_and_material_readonly_including_static() {
    let mut state = test_state();
    let (entity, _) = cube_and_camera(&state);
    let material = ResourceId::from_stable_label("res://physics/test-material");
    state.world.expect_with_world_mut(|scene| {
        scene
            .set_collider(
                entity,
                Some(ColliderComponent {
                    shape: ColliderShape::Sphere { radius: 2.0 },
                    sensor: true,
                    layer: 3,
                    collision_group: 5,
                    collision_mask: 13,
                    material: Some(ResourceHandle::new(material)),
                    ..Default::default()
                }),
            )
            .unwrap();
    });
    assert_readonly_details(
        &mut state,
        entity,
        "zircon_runtime::scene::components::ColliderComponent",
        "Collider",
        &[
            ("shape", ReflectedValue::Enum("Sphere".into())),
            ("sensor", ReflectedValue::Bool(true)),
            ("layer", ReflectedValue::Unsigned(3)),
            ("collision_group", ReflectedValue::Unsigned(5)),
            ("collision_mask", ReflectedValue::Unsigned(13)),
            ("material", ReflectedValue::Resource(material.to_string())),
        ],
    );
}

#[test]
fn joint_displays_existing_links_axes_and_limits_readonly_including_static() {
    let mut state = test_state();
    let (entity, other) = cube_and_camera(&state);
    state.world.expect_with_world_mut(|scene| {
        scene
            .set_joint(
                entity,
                Some(JointComponent {
                    joint_type: JointKind::Hinge,
                    connected_entity: Some(other),
                    anchor: Vec3::new(1.0, 2.0, 3.0),
                    axis: Vec3::X,
                    limits: Some([-0.5, 0.75]),
                    collide_connected: true,
                    ..Default::default()
                }),
            )
            .unwrap();
    });
    assert_readonly_details(
        &mut state,
        entity,
        "zircon_runtime::scene::components::JointComponent",
        "Joint",
        &[
            ("joint_type", ReflectedValue::Enum("Hinge".into())),
            ("connected_entity", ReflectedValue::Entity(Some(other))),
            ("anchor", ReflectedValue::Vec3([1.0, 2.0, 3.0])),
            ("axis", ReflectedValue::Vec3([1.0, 0.0, 0.0])),
            ("limits", ReflectedValue::Vec2([-0.5, 0.75])),
            ("collide_connected", ReflectedValue::Bool(true)),
        ],
    );
}

#[test]
fn post_process_displays_nested_scalars_readonly_including_static() {
    let mut state = test_state();
    let (entity, _) = cube_and_camera(&state);
    state.world.expect_with_world_mut(|scene| {
        let mut settings = PostProcessSettingsComponent::default();
        settings.ambient_occlusion.intensity = 0.25;
        settings.bloom.threshold = 2.0;
        settings.bloom.intensity = 0.5;
        settings.bloom.radius = 3.0;
        settings.color_grading.exposure = 1.5;
        scene.insert(entity, settings).unwrap();
    });
    assert_readonly_details(
        &mut state,
        entity,
        "zircon_runtime::scene::components::PostProcessSettingsComponent",
        "Post Process",
        &[
            ("ao_intensity", ReflectedValue::Scalar(0.25)),
            ("bloom_threshold", ReflectedValue::Scalar(2.0)),
            ("bloom_intensity", ReflectedValue::Scalar(0.5)),
            ("bloom_radius", ReflectedValue::Scalar(3.0)),
            ("color_grading_exposure", ReflectedValue::Scalar(1.5)),
        ],
    );
}

#[test]
fn post_process_volume_displays_flags_and_blending_readonly_including_static() {
    let mut state = test_state();
    let (entity, _) = cube_and_camera(&state);
    state.world.expect_with_world_mut(|scene| {
        scene
            .insert(
                entity,
                PostProcessVolumeComponent {
                    active: false,
                    is_global: false,
                    priority: 4.0,
                    weight: 0.5,
                    blend_distance: 2.0,
                    ..Default::default()
                },
            )
            .unwrap();
    });
    assert_readonly_details(
        &mut state,
        entity,
        "zircon_runtime::scene::components::PostProcessVolumeComponent",
        "Post Process Volume",
        &[
            ("active", ReflectedValue::Bool(false)),
            ("is_global", ReflectedValue::Bool(false)),
            ("priority", ReflectedValue::Scalar(4.0)),
            ("weight", ReflectedValue::Scalar(0.5)),
            ("blend_distance", ReflectedValue::Scalar(2.0)),
        ],
    );
}

#[test]
fn animation_player_displays_clip_and_playback_readonly_including_static() {
    let mut state = test_state();
    let (entity, _) = cube_and_camera(&state);
    let clip = ResourceId::from_stable_label("res://animations/test-clip");
    state.world.expect_with_world_mut(|scene| {
        scene
            .set_animation_player(
                entity,
                Some(AnimationPlayerComponent {
                    clip: ResourceHandle::new(clip),
                    playback_speed: 1.5,
                    time_seconds: 2.0,
                    weight: 0.75,
                    looping: true,
                    playing: false,
                }),
            )
            .unwrap();
    });
    assert_readonly_details(
        &mut state,
        entity,
        "zircon_runtime::scene::components::AnimationPlayerComponent",
        "Animation Player",
        &[
            ("clip", ReflectedValue::Resource(clip.to_string())),
            ("playback_speed", ReflectedValue::Scalar(1.5)),
            ("time_seconds", ReflectedValue::Scalar(2.0)),
            ("weight", ReflectedValue::Scalar(0.75)),
            ("looping", ReflectedValue::Bool(true)),
            ("playing", ReflectedValue::Bool(false)),
        ],
    );
}

#[test]
fn sprite2d_displays_resources_colour_size_and_flags_readonly_including_static() {
    let mut state = test_state();
    let (entity, _) = cube_and_camera(&state);
    let image = ResourceId::from_stable_label("res://sprites/test-image");
    let material = ResourceId::from_stable_label("res://sprites/test-material");
    state.world.expect_with_world_mut(|scene| {
        scene
            .insert(
                entity,
                Sprite2dComponent {
                    image: ResourceHandle::new(image),
                    material: Some(ResourceHandle::new(material)),
                    flip_x: true,
                    flip_y: false,
                    custom_size: Some(Vec2::new(2.0, 3.0)),
                    color: Vec4::new(0.25, 0.5, 0.75, 1.0),
                    z_order: -3,
                    ..Default::default()
                },
            )
            .unwrap();
    });
    assert_readonly_details(
        &mut state,
        entity,
        "zircon_runtime::scene::components::Sprite2dComponent",
        "Sprite 2D",
        &[
            ("image", ReflectedValue::Resource(image.to_string())),
            ("material", ReflectedValue::Resource(material.to_string())),
            ("flip_x", ReflectedValue::Bool(true)),
            ("flip_y", ReflectedValue::Bool(false)),
            ("custom_size", ReflectedValue::Vec2([2.0, 3.0])),
            ("color", ReflectedValue::Vec4([0.25, 0.5, 0.75, 1.0])),
            ("z_order", ReflectedValue::Integer(-3)),
        ],
    );
}
