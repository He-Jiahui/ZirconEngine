use super::*;

#[test]
fn world_spawn_insert_get_mut_and_remove_typed_components() {
    let mut world = World::empty();
    assert!(!world.contains_component::<Health>(u64::MAX));
    assert!(!world.is_component_added::<Health>(u64::MAX));
    assert!(!world.is_component_changed::<Health>(u64::MAX));

    let entity = world
        .spawn((
            Name("Typed Entity".to_string()),
            Health(7),
            LocalTransform {
                transform: Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            },
        ))
        .unwrap();

    assert!(world.contains_component::<Health>(entity));
    assert!(world.contains_component::<Name>(entity));
    assert!(world.is_component_added::<Health>(entity));
    assert!(world.is_component_changed::<Health>(entity));
    assert!(world.is_component_added::<Name>(entity));
    assert!(world.is_component_changed::<Name>(entity));
    assert_eq!(world.get::<Name>(entity).unwrap().0, "Typed Entity");
    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert_eq!(
        world
            .get::<LocalTransform>(entity)
            .unwrap()
            .transform
            .translation,
        Vec3::new(2.0, 0.0, 0.0)
    );

    world.clear_trackers();
    assert!(!world.is_component_added::<Health>(entity));
    assert!(!world.is_component_changed::<Health>(entity));
    assert!(!world.is_component_added::<Name>(entity));
    assert!(!world.is_component_changed::<Name>(entity));

    world.get_mut::<Health>(entity).unwrap().0 += 5;
    assert!(!world.is_component_added::<Health>(entity));
    assert!(world.is_component_changed::<Health>(entity));
    assert!(!world.is_component_changed::<Name>(entity));

    assert_eq!(world.insert(entity, Health(3)).unwrap(), Some(Health(12)));
    assert!(!world.is_component_added::<Health>(entity));
    assert!(world.is_component_changed::<Health>(entity));

    assert_eq!(world.remove::<Health>(entity).unwrap(), Some(Health(3)));
    assert!(!world.contains_component::<Health>(entity));
    assert!(!world.is_component_added::<Health>(entity));
    assert!(!world.is_component_changed::<Health>(entity));
    assert_eq!(world.get::<Health>(entity), None);

    world.clear_trackers();
    assert_eq!(world.insert(entity, Health(99)).unwrap(), None);
    assert!(world.contains_component::<Health>(entity));
    assert!(world.is_component_added::<Health>(entity));
    assert!(world.is_component_changed::<Health>(entity));

    world.clear_trackers();
    world
        .get_mut::<Name>(entity)
        .unwrap()
        .0
        .push_str(" Renamed");
    assert!(!world.is_component_added::<Name>(entity));
    assert!(world.is_component_changed::<Name>(entity));
}

#[test]
fn typed_local_transform_insertion_rejects_values_that_cannot_be_persisted() {
    let mut world = World::new();
    let entity = world.spawn_node(crate::scene::NodeKind::Mesh);
    let original = world
        .get::<LocalTransform>(entity)
        .expect("mesh nodes must have a local transform")
        .transform;
    let mut invalid = original;
    invalid.scale.z = 0.0;

    assert!(matches!(
        world.insert(entity, LocalTransform { transform: invalid }),
        Err(SceneError::ZeroScaleTransform { entity: error_entity, axis: "z" })
            if error_entity == entity
    ));
    assert_eq!(
        world.get::<LocalTransform>(entity).unwrap().transform,
        original
    );
}

#[test]
fn world_typed_mutation_errors_report_missing_entities_as_scene_errors() {
    let mut world = World::empty();
    let missing = u64::MAX;

    assert_eq!(
        world.insert(missing, Health(1)),
        Err(SceneError::MissingEntity {
            operation: "insert component on",
            entity: missing
        })
    );
    assert_eq!(
        world.insert_bundle(missing, (Health(2),)),
        Err(SceneError::MissingEntity {
            operation: "insert component on",
            entity: missing
        })
    );
    assert_eq!(
        world.remove::<Health>(missing),
        Err(SceneError::MissingEntity {
            operation: "remove component from",
            entity: missing
        })
    );
}

#[test]
fn dynamic_component_mutation_errors_report_scene_error_variants() {
    let mut world = World::empty();
    let missing = u64::MAX;

    assert_eq!(
        world.set_dynamic_component(missing, "weather.cloud", json!({})),
        Err(SceneError::MissingEntity {
            operation: "attach dynamic component to",
            entity: missing
        })
    );
    assert_eq!(
        world.register_component_type(ComponentTypeDescriptor::new("cloud", "weather", "Cloud")),
        Err(SceneError::ComponentTypePluginPrefixMismatch {
            type_id: "cloud".to_string(),
            plugin_id: "weather".to_string()
        })
    );

    let entity = world.spawn((Name("Dynamic Entity".to_string()),)).unwrap();
    world
        .register_component_type(ComponentTypeDescriptor::new(
            "weather.cloud",
            "weather",
            "Cloud",
        ))
        .unwrap();

    assert_eq!(
        world.set_dynamic_component(entity, "weather.rain", json!({})),
        Err(SceneError::UnregisteredDynamicComponentType {
            component_id: "weather.rain".to_string()
        })
    );
}
