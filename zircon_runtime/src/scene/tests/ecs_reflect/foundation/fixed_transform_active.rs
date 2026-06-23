use super::*;

#[test]
fn active_self_reflection_write_marks_active_dirty_state() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    world.flush_pending_scene_systems();
    assert!(!world.has_pending_scene_systems());
    assert_eq!(world.active_in_hierarchy(entity), Some(true));

    let response = world
        .reflect_write(ReflectWriteRequest::new(
            fixed_component_address(entity, "ActiveSelf"),
            "value",
            ReflectedValue::Bool(false),
        ))
        .expect("active state should be writable");

    assert!(response.changed);
    assert_eq!(world.get::<ActiveSelf>(entity), Some(&ActiveSelf(false)));
    assert!(world.has_pending_scene_systems());
    assert_eq!(world.active_in_hierarchy(entity), Some(false));
}

#[test]
fn local_transform_reflection_write_marks_transform_dirty_state() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    world.flush_pending_scene_systems();
    assert!(!world.has_pending_scene_systems());

    let response = world
        .reflect_write(ReflectWriteRequest::new(
            fixed_component_address(entity, "LocalTransform"),
            "translation",
            ReflectedValue::Vec3([5.0, 6.0, 7.0]),
        ))
        .expect("local transform translation should be writable");

    assert!(response.changed);
    assert_eq!(
        world
            .get::<LocalTransform>(entity)
            .unwrap()
            .transform
            .translation,
        Vec3::new(5.0, 6.0, 7.0)
    );
    assert!(world.has_pending_scene_systems());
    assert_eq!(
        world.world_transform(entity).unwrap().translation,
        Vec3::new(5.0, 6.0, 7.0)
    );

    let scale_response = world
        .reflect_write(ReflectWriteRequest::new(
            fixed_component_address(entity, "LocalTransform"),
            "scale",
            ReflectedValue::Vec3([2.0, 3.0, 4.0]),
        ))
        .expect("local transform scale should be writable");

    assert!(scale_response.changed);
    assert_eq!(
        world.get::<LocalTransform>(entity).unwrap().transform.scale,
        Vec3::new(2.0, 3.0, 4.0)
    );
    world.flush_pending_scene_systems();
    assert!(!world.has_pending_scene_systems());
    let no_op_scale = world
        .reflect_write(ReflectWriteRequest::new(
            fixed_component_address(entity, "LocalTransform"),
            "scale",
            ReflectedValue::Vec3([2.0, 3.0, 4.0]),
        ))
        .expect("same local transform scale should be accepted as unchanged");
    assert!(!no_op_scale.changed);
    assert!(!world.has_pending_scene_systems());
    assert!(matches!(
        world.reflect_write(ReflectWriteRequest::new(
            fixed_component_address(entity, "LocalTransform"),
            "translation",
            ReflectedValue::Vec3([f32::NAN, 0.0, 0.0]),
        )),
        Err(ReflectError::TypeMismatch { expected, .. }) if expected == "finite Vec3"
    ));
    assert!(!world.has_pending_scene_systems());
}

#[test]
fn local_transform_rotation_is_readable_but_not_writable_in_m8() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh);
    world
        .insert(
            entity,
            LocalTransform {
                transform: Transform::default(),
            },
        )
        .unwrap();
    let address = fixed_component_address(entity, "LocalTransform");

    assert_eq!(
        world
            .reflect_read(ReflectReadRequest::new(address.clone(), "rotation"))
            .expect("rotation should be readable")
            .field,
        ReflectFieldValue::new("rotation", ReflectedValue::Vec4([0.0, 0.0, 0.0, 1.0]))
    );
    assert!(
        !world
            .reflect_schema("LocalTransform")
            .expect("schema should resolve by short type path")
            .type_info
            .fields
            .iter()
            .find(|field| field.name == "rotation")
            .expect("rotation schema should exist")
            .editable
    );
    assert_eq!(
        world
            .reflect_write(ReflectWriteRequest::new(
                address,
                "rotation",
                ReflectedValue::Vec4([0.0, 0.0, 0.0, 1.0]),
            ))
            .expect_err("rotation writes are deferred until a later milestone"),
        ReflectError::NonEditableField {
            type_path: "zircon_runtime::scene::components::LocalTransform".to_string(),
            field_name: "rotation".to_string(),
        }
    );
}
