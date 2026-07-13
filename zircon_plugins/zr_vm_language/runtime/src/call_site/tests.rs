use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::json;
use zircon_runtime::core::framework::scene::ComponentTypeDescriptor;
use zircon_runtime::scene::{
    EntityId, NodeKind, ReflectComponent, RuntimeTypeRegistration, TypeRegistry, World,
};
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectFieldValue,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
    ReflectedValue,
};

use super::ScriptCallTable;

#[test]
fn call_site_resolution_happens_once() {
    let mut world = World::empty();
    world
        .register_component_type(
            ComponentTypeDescriptor::new("gameplay.Component.Health", "gameplay", "Health")
                .with_property("current", "Scalar", true)
                .with_property("maximum", "Scalar", false),
        )
        .expect("dynamic component type should register");
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .set_dynamic_component(
            entity,
            "gameplay.Component.Health",
            json!({ "current": 25.0, "maximum": 100.0 }),
        )
        .expect("dynamic component should attach");

    let table = ScriptCallTable::compile(world.type_registry())
        .expect("registered reflection schema should compile into dense call sites");
    let site = table
        .resolve("gameplay.Component.Health", "current")
        .expect("field call site should resolve during module loading");

    assert_eq!(site.type_slot, 0);
    assert_eq!(site.member_slot, 0);
    assert_eq!(site.layout.value_type_path.as_ref(), "Scalar");
    assert_eq!(table.resolution_count(), 1);
    assert_eq!(
        table
            .read(&site, &world, entity)
            .expect("compiled field read should succeed"),
        ReflectedValue::Scalar(25.0)
    );
    assert!(table
        .write(&site, &mut world, entity, ReflectedValue::Scalar(40.0))
        .expect("compiled field write should succeed"));
    assert_eq!(
        table
            .read(&site, &world, entity)
            .expect("compiled field read should observe the write"),
        ReflectedValue::Scalar(40.0)
    );
    assert_eq!(
        table.resolution_count(),
        1,
        "runtime calls must not repeat type/member name resolution"
    );
}

static NAMED_FIELD_CALLS: AtomicUsize = AtomicUsize::new(0);
static DENSE_FIELD_CALLS: AtomicUsize = AtomicUsize::new(0);

#[test]
fn runtime_calls_use_dense_slots_without_field_name_dispatch() {
    NAMED_FIELD_CALLS.store(0, Ordering::Relaxed);
    DENSE_FIELD_CALLS.store(0, Ordering::Relaxed);
    let mut registry = TypeRegistry::default();
    registry
        .register(RuntimeTypeRegistration {
            registration: ReflectTypeRegistration::new(
                ReflectTypePath::new("test.DenseProbe", "DenseProbe")
                    .expect("test reflection path should be valid"),
                "DenseProbe",
                ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
                    "value",
                    "Unsigned",
                    ReflectEditorHint::Unsigned,
                )]),
                ReflectSerializationStrategy::Value,
            )
            .as_component(),
            component: Some(
                ReflectComponent::new(
                    "test.DenseProbe",
                    dense_probe_contains,
                    dense_probe_named_read,
                    dense_probe_read_fields,
                    dense_probe_named_write,
                    dense_probe_remove,
                )
                .with_dense_field_slots(dense_probe_slot_read, dense_probe_slot_write),
            ),
            resource: None,
        })
        .expect("dense probe registration should succeed");

    let table = ScriptCallTable::compile(&registry)
        .expect("dense probe should compile into one numeric call site");
    let site = table
        .resolve("test.DenseProbe", "value")
        .expect("dense probe field should resolve once during loading");
    let mut world = World::empty();

    assert_eq!(
        table
            .read(&site, &world, 7)
            .expect("dense probe read should use the slot callback"),
        ReflectedValue::Unsigned(0)
    );
    assert!(table
        .write(&site, &mut world, 7, ReflectedValue::Unsigned(1))
        .expect("dense probe write should use the slot callback"));
    assert_eq!(NAMED_FIELD_CALLS.load(Ordering::Relaxed), 0);
    assert_eq!(DENSE_FIELD_CALLS.load(Ordering::Relaxed), 2);
}

fn dense_probe_contains(_world: &World, _entity: EntityId, _type_path: &str) -> bool {
    true
}

fn dense_probe_named_read(
    _world: &World,
    _entity: EntityId,
    _type_path: &str,
    _field_name: &str,
) -> Result<ReflectedValue, ReflectError> {
    NAMED_FIELD_CALLS.fetch_add(1, Ordering::Relaxed);
    Ok(ReflectedValue::Unsigned(u64::MAX))
}

fn dense_probe_read_fields(
    _world: &World,
    _entity: EntityId,
    _type_path: &str,
) -> Result<Vec<ReflectFieldValue>, ReflectError> {
    Ok(Vec::new())
}

fn dense_probe_named_write(
    _world: &mut World,
    _entity: EntityId,
    _type_path: &str,
    _field_name: &str,
    _value: ReflectedValue,
) -> Result<bool, ReflectError> {
    NAMED_FIELD_CALLS.fetch_add(1, Ordering::Relaxed);
    Ok(false)
}

fn dense_probe_slot_read(
    _world: &World,
    _entity: EntityId,
    _type_path: &str,
    field_slot: u32,
) -> Result<ReflectedValue, ReflectError> {
    DENSE_FIELD_CALLS.fetch_add(1, Ordering::Relaxed);
    Ok(ReflectedValue::Unsigned(u64::from(field_slot)))
}

fn dense_probe_slot_write(
    _world: &mut World,
    _entity: EntityId,
    _type_path: &str,
    field_slot: u32,
    _value: ReflectedValue,
) -> Result<bool, ReflectError> {
    DENSE_FIELD_CALLS.fetch_add(1, Ordering::Relaxed);
    Ok(field_slot == 0)
}

fn dense_probe_remove(
    _world: &mut World,
    _entity: EntityId,
    _type_path: &str,
) -> Result<bool, ReflectError> {
    Ok(false)
}
