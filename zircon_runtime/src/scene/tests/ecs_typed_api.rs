use serde_json::json;

use crate::core::math::{Transform, Vec3};
use crate::plugin::{ComponentPropertyDescriptor, ComponentTypeDescriptor};
use crate::scene::components::{LocalTransform, Name, RenderLayerMask, RigidBodyComponent};
use crate::scene::ecs::{Component, Resource};
use crate::scene::World;

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct FrameCounter(u32);

impl Resource for FrameCounter {}

#[test]
fn world_spawn_insert_get_mut_and_remove_typed_components() {
    let mut world = World::empty();
    let entity = world
        .spawn((
            Name("Typed Entity".to_string()),
            Health(7),
            LocalTransform {
                transform: Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            },
        ))
        .unwrap();

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

    world.get_mut::<Health>(entity).unwrap().0 += 5;

    assert_eq!(world.remove::<Health>(entity).unwrap(), Some(Health(12)));
    assert_eq!(world.get::<Health>(entity), None);
}

#[test]
fn world_resources_are_registered_and_replaced_by_type() {
    let mut world = World::empty();

    let resource_id = world.resource_id::<FrameCounter>();
    assert_eq!(resource_id.index(), 0);
    assert_eq!(world.insert_resource(FrameCounter(1)), None);
    assert_eq!(world.resource::<FrameCounter>(), &FrameCounter(1));
    world.resource_mut::<FrameCounter>().0 += 1;

    assert_eq!(
        world.insert_resource(FrameCounter(9)),
        Some(FrameCounter(2))
    );
    assert_eq!(world.resource::<FrameCounter>(), &FrameCounter(9));
    assert_eq!(
        world.registered_resource_id::<FrameCounter>(),
        Some(resource_id)
    );
}

#[test]
fn fixed_component_setters_and_dynamic_components_share_component_id_presence() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Runtime Body".to_string()),)).unwrap();
    let rigid_body_id = world.component_id::<RigidBodyComponent>();

    world
        .set_rigid_body(entity, Some(RigidBodyComponent::default()))
        .unwrap();

    assert!(world.contains_component_id(entity, rigid_body_id));
    assert_eq!(
        world.get::<RigidBodyComponent>(entity),
        Some(&RigidBodyComponent::default())
    );

    world.set_rigid_body(entity, None).unwrap();

    assert!(!world.contains_component_id(entity, rigid_body_id));
    assert_eq!(world.get::<RigidBodyComponent>(entity), None);

    world
        .register_component_type(ComponentTypeDescriptor {
            type_id: "weather.cloud".to_string(),
            plugin_id: "weather".to_string(),
            display_name: "Cloud".to_string(),
            properties: vec![ComponentPropertyDescriptor {
                name: "density".to_string(),
                value_type: "number".to_string(),
                editable: true,
            }],
        })
        .unwrap();

    world
        .set_dynamic_component(entity, "weather.cloud", json!({ "density": 0.75 }))
        .unwrap();
    let dynamic_component_id = world
        .registered_dynamic_component_id("weather.cloud")
        .unwrap();

    assert!(world.contains_component_id(entity, dynamic_component_id));
    assert_eq!(
        world.dynamic_component(entity, "weather.cloud"),
        Some(&json!({ "density": 0.75 }))
    );

    world
        .remove_dynamic_component(entity, "weather.cloud")
        .unwrap();

    assert!(!world.contains_component_id(entity, dynamic_component_id));
}

#[test]
fn runtime_only_typed_ecs_state_is_not_serialized() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Serialized Entity".to_string()), Health(42)))
        .unwrap();
    world.insert_resource(FrameCounter(3));

    let saved = serde_json::to_string(&world).unwrap();
    let mut loaded: World = serde_json::from_str(&saved).unwrap();

    assert!(!saved.contains("FrameCounter"));
    assert_eq!(loaded.get::<Health>(entity), None);
    assert_eq!(loaded.get_resource::<FrameCounter>(), None);
    assert_eq!(
        loaded.get::<Name>(entity),
        Some(&Name("Serialized Entity".to_string()))
    );
    let name_component_id = loaded.component_id::<Name>();
    let render_layer_mask_component_id = loaded.component_id::<RenderLayerMask>();

    assert!(loaded.contains_component_id(entity, name_component_id));
    assert!(loaded.contains_component_id(entity, render_layer_mask_component_id));
}
