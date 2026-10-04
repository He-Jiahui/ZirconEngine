use serde_json::json;
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::scene::{NodeKind, World};

use super::{reflect_component_for_dynamic_descriptor, registration_from_component_descriptor};

#[test]
fn dense_batch_write_publishes_one_dynamic_component_mutation() {
    let descriptor = ComponentTypeDescriptor::new(
        "runtime.tests.DynamicBatchComponent",
        "runtime.tests",
        "Dynamic Batch Component",
    )
    .with_property("first", "Scalar", true)
    .with_property("second", "Scalar", true);
    let mut world = World::empty();
    world
        .register_component_type(descriptor.clone())
        .expect("test dynamic component descriptor must register");
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            &descriptor.type_id,
            json!({"first": 1.0, "second": 2.0}),
        )
        .expect("test dynamic component must attach");

    let generation_before = world.world_generation();
    reflect_component_for_dynamic_descriptor(&descriptor)
        .write_fields_by_slot(
            &mut world,
            entity,
            vec![
                (0, ReflectedValue::Scalar(3.0)),
                (1, ReflectedValue::Scalar(4.0)),
            ],
        )
        .expect("batch field write must succeed");

    assert_eq!(world.world_generation(), generation_before + 1);
    assert_eq!(
        world.dynamic_component(entity, &descriptor.type_id),
        Some(&json!({"first": 3.0, "second": 4.0}))
    );
}

#[test]
fn dynamic_component_stage_clone_preserves_json_in_preflight_world() {
    let descriptor = ComponentTypeDescriptor::new(
        "runtime.tests.DynamicStagedComponent",
        "runtime.tests",
        "Dynamic Staged Component",
    )
    .with_property("value", "Scalar", true);
    let unselected_descriptor = ComponentTypeDescriptor::new(
        "runtime.tests.DynamicUnselectedComponent",
        "runtime.tests",
        "Dynamic Unselected Component",
    )
    .with_property("value", "Scalar", true);
    let mut source = World::empty();
    let selected_registration = registration_from_component_descriptor(&descriptor)
        .expect("selected VM registration must derive from its descriptor");
    source
        .sync_vm_types(std::slice::from_ref(&selected_registration))
        .expect("source VM catalog must register the selected descriptor");
    source
        .register_component_type(unselected_descriptor.clone())
        .expect("unselected source dynamic component descriptor must register");
    let entity = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    source
        .set_dynamic_component(entity, &descriptor.type_id, json!({"value": 5.0}))
        .expect("source dynamic component must attach");
    source
        .set_dynamic_component(
            entity,
            &unselected_descriptor.type_id,
            json!({"value": 9.0}),
        )
        .expect("unselected source dynamic component must attach");

    let mut preflight = source.dynamic_scene_preflight_world([
        selected_registration.type_path.short_type_path(),
        descriptor.type_id.as_str(),
    ]);
    assert!(preflight
        .component_type_descriptor(&descriptor.type_id)
        .is_some());
    assert!(preflight.type_registry().contains(&descriptor.type_id));
    assert!(preflight
        .component_type_descriptor(&unselected_descriptor.type_id)
        .is_none());
    assert!(!preflight
        .type_registry()
        .contains(&unselected_descriptor.type_id));
    preflight
        .insert_owned_node_records(vec![source
            .node_record(entity)
            .expect("source node record")])
        .expect("preflight identity must be restored before component staging");

    source
        .stage_reflected_component_clone(entity, &descriptor.type_id, &mut preflight)
        .expect("dynamic component staging must succeed");
    assert_eq!(
        preflight.dynamic_component(entity, &descriptor.type_id),
        Some(&json!({"value": 5.0}))
    );
    assert_eq!(
        preflight.dynamic_component(entity, &unselected_descriptor.type_id),
        None
    );
}
