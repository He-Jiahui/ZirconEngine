use serde_json::json;
use zircon_runtime_interface::reflect::ReflectedValue;

use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::scene::{NodeKind, World};

use super::{reflect_component_for_dynamic_descriptor, registration_from_component_descriptor};

/// 同批更新两个字段只发布一次完整动态组件变更，同时核对 JSON 与 World 代数。
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

/// 先恢复稳定实体身份，再暂存所选类型的 JSON 副本；未选类型及其目录项不进入预检 World。
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
        .sync_vm_types(
            std::slice::from_ref(&selected_registration),
            &Default::default(),
        )
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

fn declared_structure_fixture() -> (
    World,
    crate::scene::EntityId,
    crate::scene::reflect::ReflectComponent,
) {
    let descriptor = ComponentTypeDescriptor::new(
        "runtime.tests.DeclaredStructures",
        "runtime.tests",
        "Declared Structures",
    )
    .with_property("literal", "Json", true)
    .with_property("reference", "Entity", true)
    .with_property("references", "List<Entity>", true)
    .with_property("named", "Map<String, Entity>", true);
    let mut world = World::empty();
    world.register_component_type(descriptor.clone()).unwrap();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .set_dynamic_component(
            entity,
            &descriptor.type_id,
            json!({
                "literal": {"nested": [{"entity": 77}, {"entity": null}]},
                "reference": {"entity": entity},
                "references": [{"entity": entity}, {"entity": null}],
                "named": {"self": {"entity": entity}, "none": {"entity": null}},
            }),
        )
        .unwrap();
    let adapter = reflect_component_for_dynamic_descriptor(&descriptor);
    (world, entity, adapter)
}

#[test]
fn non_vm_declared_structures_roundtrip_named_single_and_dense_batch_fields() {
    let (mut world, entity, adapter) = declared_structure_fixture();
    let literal = json!({"nested": [{"entity": 77}, {"entity": null}]});
    let named = std::collections::BTreeMap::from([
        ("none".to_string(), ReflectedValue::Entity(None)),
        ("self".to_string(), ReflectedValue::Entity(Some(entity))),
    ]);
    let expected = [
        ("literal", ReflectedValue::Json(literal)),
        ("reference", ReflectedValue::Entity(Some(entity))),
        (
            "references",
            ReflectedValue::List(vec![
                ReflectedValue::Entity(Some(entity)),
                ReflectedValue::Entity(None),
            ]),
        ),
        ("named", ReflectedValue::Map(named)),
    ];
    for (slot, (name, value)) in expected.iter().enumerate() {
        assert_eq!(adapter.read_field(&world, entity, name).unwrap(), *value);
        assert_eq!(
            adapter
                .read_field_by_slot(&world, entity, slot as u32)
                .unwrap(),
            *value
        );
    }

    let replacements = [
        (
            "literal",
            ReflectedValue::Json(json!({"entity": 91, "nested": [null, {"entity": 92}]})),
        ),
        ("reference", ReflectedValue::Entity(None)),
        (
            "references",
            ReflectedValue::List(vec![ReflectedValue::Entity(None)]),
        ),
        (
            "named",
            ReflectedValue::Map(std::collections::BTreeMap::from([(
                "next".to_string(),
                ReflectedValue::Entity(Some(entity)),
            )])),
        ),
    ];
    for (name, value) in &replacements {
        assert!(adapter
            .write_field(&mut world, entity, name, value.clone())
            .unwrap());
        assert_eq!(adapter.read_field(&world, entity, name).unwrap(), *value);
    }
    assert_eq!(
        world.dynamic_component(entity, "runtime.tests.DeclaredStructures"),
        Some(&json!({
            "literal": {"entity": 91, "nested": [null, {"entity": 92}]},
            "reference": {"entity": null},
            "references": [{"entity": null}],
            "named": {"next": {"entity": entity}},
        }))
    );

    for (slot, (name, value)) in expected.iter().enumerate() {
        assert!(adapter
            .write_field_by_slot(&mut world, entity, slot as u32, value.clone())
            .unwrap());
        assert_eq!(adapter.read_field(&world, entity, name).unwrap(), *value);
    }

    let generation = world.world_generation();
    assert!(adapter
        .write_fields_by_slot(
            &mut world,
            entity,
            replacements
                .iter()
                .enumerate()
                .map(|(slot, (_, value))| (slot as u32, value.clone()))
                .collect(),
        )
        .unwrap());
    assert_eq!(world.world_generation(), generation + 1);
    for (slot, (name, value)) in replacements.iter().enumerate() {
        assert_eq!(adapter.read_field(&world, entity, name).unwrap(), *value);
        assert_eq!(
            adapter
                .read_field_by_slot(&world, entity, slot as u32)
                .unwrap(),
            *value
        );
    }
}

#[test]
fn non_vm_declared_structure_type_errors_preserve_payload_and_generation() {
    let (mut world, entity, adapter) = declared_structure_fixture();
    let before = world
        .dynamic_component(entity, "runtime.tests.DeclaredStructures")
        .unwrap()
        .clone();
    let generation = world.world_generation();
    let invalid = [
        ("literal", ReflectedValue::Entity(Some(entity))),
        ("reference", ReflectedValue::Json(json!({"entity": entity}))),
        (
            "references",
            ReflectedValue::List(vec![ReflectedValue::Json(json!({"entity": entity}))]),
        ),
        (
            "named",
            ReflectedValue::Map(std::collections::BTreeMap::from([(
                "wrong".to_string(),
                ReflectedValue::Integer(7),
            )])),
        ),
    ];
    for (slot, (name, value)) in invalid.iter().enumerate() {
        let error = adapter
            .write_field(&mut world, entity, name, value.clone())
            .expect_err("named writes must reject the wrong reflected structure");
        assert!(matches!(
            error,
            zircon_runtime_interface::reflect::ReflectError::TypeMismatch {
                ref field_name, ..
            } if field_name.as_str() == *name
        ));
        let error = adapter
            .write_field_by_slot(&mut world, entity, slot as u32, value.clone())
            .expect_err("single-slot writes must reject the wrong reflected structure");
        assert!(matches!(
            error,
            zircon_runtime_interface::reflect::ReflectError::TypeMismatch {
                ref field_name, ..
            } if field_name.as_str() == *name
        ));
        assert_eq!(world.world_generation(), generation);
        assert_eq!(
            world.dynamic_component(entity, "runtime.tests.DeclaredStructures"),
            Some(&before)
        );
    }

    let error = adapter
        .write_fields_by_slot(
            &mut world,
            entity,
            vec![
                (0, ReflectedValue::Json(json!({"changed": true}))),
                (
                    2,
                    ReflectedValue::List(vec![ReflectedValue::Json(json!({"entity": entity}))]),
                ),
            ],
        )
        .expect_err("a later invalid slot must not publish the earlier valid candidate");
    assert!(matches!(
        error,
        zircon_runtime_interface::reflect::ReflectError::TypeMismatch {
            ref field_name, ..
        } if field_name == "references"
    ));
    assert_eq!(world.world_generation(), generation);
    assert_eq!(
        world.dynamic_component(entity, "runtime.tests.DeclaredStructures"),
        Some(&before)
    );
}
