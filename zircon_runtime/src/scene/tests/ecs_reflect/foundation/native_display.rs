use super::*;
use crate::core::resource::{ResourceHandle, ResourceId};
use crate::scene::components::{
    AnimationPlayerComponent, ColliderComponent, ColliderShape, JointComponent, JointKind,
    PostProcessSettingsComponent, PostProcessVolumeComponent, Sprite2dComponent,
};
use zircon_runtime_interface::reflect::ReflectFieldsRequest;

fn read(world: &World, entity: EntityId, component: &str, field: &str) -> ReflectedValue {
    let address = fixed_component_address(
        entity,
        &format!("zircon_runtime::scene::components::{component}"),
    );
    world
        .reflect_read(ReflectReadRequest::new(
            address.clone(),
            reflected_field_id(address.type_path(), field),
        ))
        .unwrap()
        .field
        .value
}

#[test]
fn new_native_display_registrations_are_readonly_and_reject_direct_writes() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh).unwrap();
    world.insert(entity, ColliderComponent::default()).unwrap();
    world.insert(entity, JointComponent::default()).unwrap();
    world
        .insert(entity, PostProcessSettingsComponent::default())
        .unwrap();
    world
        .insert(entity, PostProcessVolumeComponent::default())
        .unwrap();
    world
        .insert(
            entity,
            AnimationPlayerComponent {
                clip: ResourceHandle::new(ResourceId::from_stable_label("res://test/animation")),
                playback_speed: 1.0,
                time_seconds: 0.0,
                weight: 1.0,
                looping: false,
                playing: true,
            },
        )
        .unwrap();
    world.insert(entity, Sprite2dComponent::default()).unwrap();
    for (component, count) in [
        ("ColliderComponent", 6),
        ("JointComponent", 6),
        ("PostProcessSettingsComponent", 5),
        ("PostProcessVolumeComponent", 5),
        ("AnimationPlayerComponent", 6),
        ("Sprite2dComponent", 7),
    ] {
        let path = format!("zircon_runtime::scene::components::{component}");
        let schema = world.reflect_schema(&path).unwrap();
        assert_eq!(schema.type_info.fields.len(), count);
        assert!(schema
            .type_info
            .fields
            .iter()
            .all(|field| field.editor_visible && !field.editable));
        let address = fixed_component_address(entity, component);
        let original = world
            .reflect_fields(ReflectFieldsRequest::new(address.clone()))
            .unwrap();
        for field in &original.fields {
            assert!(
                matches!(
                    world.reflect_write(ReflectWriteRequest::new(
                        address.clone(),
                        field.field_id,
                        field.value.clone(),
                    )),
                    Err(ReflectError::NonEditableField { .. })
                ),
                "{component}.{}",
                field.field_name
            );
        }
        assert_eq!(
            world
                .reflect_fields(ReflectFieldsRequest::new(address))
                .unwrap()
                .fields,
            original.fields
        );
        assert!(!world
            .dynamic_component_creation_descriptors()
            .any(|entry| entry.type_id == path));
    }
    assert_eq!(
        read(&world, entity, "ColliderComponent", "material"),
        ReflectedValue::Null
    );
    assert_eq!(
        read(&world, entity, "JointComponent", "connected_entity"),
        ReflectedValue::Entity(None)
    );
    assert_eq!(
        read(&world, entity, "JointComponent", "limits"),
        ReflectedValue::Null
    );
    assert_eq!(
        read(&world, entity, "Sprite2dComponent", "material"),
        ReflectedValue::Null
    );
    assert_eq!(
        read(&world, entity, "Sprite2dComponent", "custom_size"),
        ReflectedValue::Null
    );
}

#[test]
fn collider_shape_display_reads_discriminants_without_geometry_payloads() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh).unwrap();
    for (shape, name) in [
        (ColliderShape::default(), "Box"),
        (ColliderShape::Sphere { radius: 2.0 }, "Sphere"),
        (
            ColliderShape::Capsule {
                radius: 1.0,
                half_height: 2.0,
            },
            "Capsule",
        ),
        (
            ColliderShape::Cylinder {
                radius: 1.0,
                half_height: 2.0,
            },
            "Cylinder",
        ),
        (
            ColliderShape::ConvexHull {
                points: vec![Vec3::ZERO],
            },
            "ConvexHull",
        ),
        (ColliderShape::Compound { children: vec![] }, "Compound"),
    ] {
        world
            .set_collider(
                entity,
                Some(ColliderComponent {
                    shape,
                    ..Default::default()
                }),
            )
            .unwrap();
        assert_eq!(
            read(&world, entity, "ColliderComponent", "shape"),
            ReflectedValue::Enum(name.into())
        );
    }
}

#[test]
fn joint_display_reads_each_kind_without_selecting_constraint_policy() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Mesh).unwrap();
    for (joint_type, name) in [
        (JointKind::Fixed, "Fixed"),
        (JointKind::Distance, "Distance"),
        (JointKind::Hinge, "Hinge"),
        (JointKind::Slider, "Slider"),
        (JointKind::ConeTwist, "ConeTwist"),
        (JointKind::Generic6Dof, "Generic6Dof"),
    ] {
        world
            .set_joint(
                entity,
                Some(JointComponent {
                    joint_type,
                    ..Default::default()
                }),
            )
            .unwrap();
        assert_eq!(
            read(&world, entity, "JointComponent", "joint_type"),
            ReflectedValue::Enum(name.into())
        );
    }
}
