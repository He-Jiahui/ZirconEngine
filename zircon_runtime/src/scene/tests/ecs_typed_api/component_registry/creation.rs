use super::*;
use crate::scene::world::{DynamicComponentCreationError, DynamicComponentCreationSpec};
use crate::scene::{EntityId, World};
use serde_json::json;

const TYPE: &str = "weather.Component.CloudLayer";

fn descriptor() -> ComponentTypeDescriptor {
    ComponentTypeDescriptor::new(TYPE, "weather", "Cloud Layer")
        .with_property("coverage", "Scalar", true)
        .with_property("label", "String", false)
}
fn fixture() -> (World, Vec<EntityId>, DynamicComponentCreationSpec) {
    let mut world = World::empty();
    world.register_component_type(descriptor()).unwrap();
    let targets = vec![
        world.spawn((Name("A".into()),)).unwrap(),
        world.spawn((Name("B".into()),)).unwrap(),
    ];
    let mut spec = world
        .dynamic_component_creation_template(TYPE)
        .unwrap()
        .spec;
    spec.value = json!({"coverage":0.25,"label":"thin"});
    (world, targets, spec)
}

#[test]
fn dynamic_creation_preflights_missing_targets_and_complete_values_before_batch_publication() {
    let (mut world, targets, mut spec) = fixture();
    let generation = world.world_generation();
    assert!(world
        .create_dynamic_components(&[targets[0], u64::MAX], &spec)
        .is_err());
    assert!(world
        .create_dynamic_components(&[targets[0], targets[0]], &spec)
        .is_err());
    spec.value = json!({"coverage":0.25});
    assert!(world.create_dynamic_components(&targets, &spec).is_err());
    assert_eq!(world.world_generation(), generation);
    for target in targets {
        assert!(world.dynamic_component(target, TYPE).is_none());
    }
}

#[test]
fn dynamic_creation_uses_only_real_declared_defaults_and_requires_remaining_values() {
    let mut world = World::empty();
    let mut registration =
        crate::scene::reflect::registration_from_component_descriptor(&descriptor()).unwrap();
    registration
        .type_info
        .fields
        .iter_mut()
        .find(|field| field.name == "coverage")
        .unwrap()
        .default_value = Some(zircon_runtime_interface::reflect::ReflectedValue::Scalar(
        0.5,
    ));
    world
        .register_vm_type(
            registration,
            crate::scene::reflect::VmTypeBacking::DynamicComponent,
        )
        .unwrap();
    let target = world.spawn((Name("A".into()),)).unwrap();
    let template = world.dynamic_component_creation_template(TYPE).unwrap();
    assert_eq!(template.spec.value, json!({"coverage":0.5}));
    assert_eq!(template.required_fields, vec!["label"]);
    assert!(world
        .validate_dynamic_component_creation(&[target], &template.spec)
        .is_err());
    let mut spec = template.spec;
    spec.value["label"] = json!("declared");
    let receipts = world.create_dynamic_components(&[target], &spec).unwrap();
    world
        .validate_created_dynamic_components(&[target], &spec, &receipts)
        .unwrap();
}

#[test]
fn dynamic_creation_preserves_value_edit_identity_but_rejects_equal_json_recreation_and_new_world()
{
    let (mut world, targets, spec) = fixture();
    let receipts = world.create_dynamic_components(&targets, &spec).unwrap();
    assert_ne!(
        receipts[0], receipts[1],
        "each real instance needs a fresh token"
    );
    let mut edited = spec.clone();
    edited.value = json!({"coverage":0.75,"label":"edited"});
    assert!(world
        .set_dynamic_component(targets[0], TYPE, edited.value.clone())
        .unwrap());
    world
        .validate_created_dynamic_components(&[targets[0]], &edited, &receipts[..1])
        .unwrap();
    assert!(world
        .set_dynamic_component(targets[0], TYPE, spec.value.clone())
        .unwrap());
    world
        .validate_created_dynamic_components(&targets, &spec, &receipts)
        .unwrap();
    let cloned = world.clone();
    assert!(cloned
        .validate_created_dynamic_components(&targets, &spec, &receipts)
        .is_err());
    world.remove_dynamic_component(targets[1], TYPE).unwrap();
    world
        .set_dynamic_component(targets[1], TYPE, spec.value.clone())
        .unwrap();
    assert!(matches!(
        world.remove_created_dynamic_components(&targets, &spec, &receipts),
        Err(DynamicComponentCreationError::CreationChanged { .. })
    ));
    for target in targets {
        assert_eq!(world.dynamic_component(target, TYPE), Some(&spec.value));
    }
}

#[test]
fn dynamic_creation_batch_removal_and_recreation_preserve_values_without_reusing_tokens() {
    let (mut world, targets, spec) = fixture();
    let receipts = world.create_dynamic_components(&targets, &spec).unwrap();
    world
        .remove_created_dynamic_components(&targets, &spec, &receipts)
        .unwrap();
    for &target in &targets {
        assert!(world.dynamic_component(target, TYPE).is_none());
    }
    let new_receipts = world.create_dynamic_components(&targets, &spec).unwrap();
    assert!(receipts
        .iter()
        .zip(&new_receipts)
        .all(|(before, after)| before != after));
    world
        .validate_created_dynamic_components(&targets, &spec, &new_receipts)
        .unwrap();
    assert!(world
        .remove_created_dynamic_components(&targets, &spec, &receipts)
        .is_err());
}
