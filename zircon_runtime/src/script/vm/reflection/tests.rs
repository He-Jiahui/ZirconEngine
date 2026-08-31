use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectFieldId, ReflectFieldInfo, ReflectObjectAddress, ReflectReadRequest,
    ReflectScriptVisibility, ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath,
    ReflectTypeRegistration, ReflectedValue,
};

use std::collections::BTreeMap;
use std::sync::{Arc, Barrier};

use crate::core::framework::scene::{LevelManager, SCENE_MODULE_NAME};
use crate::core::{CoreRuntime, TasksModule, TASKS_MODULE_NAME};
use crate::engine_module::EngineModule;
use crate::scene::{NodeKind, SceneError, VmTypeBacking, World, WorldReflection};
use crate::script::{PluginSlotId, VmStateSchema, VmStateTypeSchema};

use super::{VmReflectionCatalog, VmReflectionError, VmReflectionSchema};

mod schema_invariants;

fn reflection_test_runtime(catalog: &VmReflectionCatalog) -> CoreRuntime {
    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::foundation::module_descriptor())
        .expect("reflection test foundation module should register");
    runtime
        .register_module(TasksModule.descriptor())
        .expect("reflection test tasks module should register");
    runtime
        .register_module(crate::asset::module_descriptor())
        .expect("reflection test asset module should register");
    runtime
        .register_module(crate::scene::module_descriptor())
        .expect("reflection test scene module should register");
    runtime
        .activate_module(crate::core::framework::foundation::FOUNDATION_MODULE_NAME)
        .expect("reflection test foundation module should activate");
    runtime
        .activate_module(TASKS_MODULE_NAME)
        .expect("reflection test tasks module should activate");
    runtime
        .activate_module(crate::asset::ASSET_MODULE_NAME)
        .expect("reflection test asset module should activate");
    runtime
        .activate_module(SCENE_MODULE_NAME)
        .expect("reflection test scene module should activate");
    catalog.bind_core(&runtime.handle());
    runtime
}

fn create_managed_level(catalog: &VmReflectionCatalog) -> (CoreRuntime, crate::scene::LevelSystem) {
    let runtime = reflection_test_runtime(catalog);
    let level = crate::scene::create_default_level(&runtime.handle())
        .expect("reflection test managed level should be created");
    (runtime, level)
}

#[test]
fn reflection_schema_projects_only_public_vm_components() {
    let schema = state_schema(vec![
        vm_component_registration(
            "gameplay.Component.PublicHealth",
            "PublicHealth",
            ReflectScriptVisibility::Public,
            vec![scalar_field("current")],
        ),
        vm_component_registration(
            "gameplay.Component.PrivateState",
            "PrivateState",
            ReflectScriptVisibility::Private,
            vec![scalar_field("value")],
        ),
    ]);

    let projected = VmReflectionSchema::from_state_schema(&schema)
        .expect("valid public VM component schema should project");

    assert_eq!(projected.registrations().len(), 1);
    assert_eq!(
        projected.registrations()[0].type_path.type_path(),
        "gameplay.Component.PublicHealth"
    );
}

#[test]
fn catalog_registers_public_vm_types_in_existing_and_future_worlds() {
    let catalog = VmReflectionCatalog::default();
    let (_runtime, existing_level) = create_managed_level(&catalog);
    let schema = state_schema(vec![vm_component_registration(
        "gameplay.Component.Health",
        "Health",
        ReflectScriptVisibility::Public,
        vec![scalar_field("current")],
    )]);

    catalog
        .publish_generation(PluginSlotId::new(1), 1, "gameplay", &schema)
        .expect("publishing a valid VM schema should update existing levels");

    existing_level.with_world(|world| {
        assert!(world
            .type_registry()
            .contains_type_path("gameplay.Component.Health"));
    });
    let mut future_world = World::empty();
    catalog
        .apply_to_world(&mut future_world)
        .expect("future worlds should receive the catalog snapshot");
    assert!(future_world
        .type_registry()
        .contains_type_path("gameplay.Component.Health"));
}

#[test]
fn manager_service_applies_catalog_to_future_worlds() {
    let catalog = VmReflectionCatalog::default();
    let runtime = reflection_test_runtime(&catalog);
    crate::scene::install_world_runtime_extension_plan(
        &runtime.handle(),
        catalog
            .world_runtime_extension_plan()
            .expect("catalog extension plan should build"),
    )
    .expect("catalog extension should install");
    catalog
        .publish_generation(
            PluginSlotId::new(11),
            1,
            "gameplay",
            &state_schema(vec![vm_component_registration(
                "gameplay.Component.FutureHealth",
                "FutureHealth",
                ReflectScriptVisibility::Public,
                vec![scalar_field("current")],
            )]),
        )
        .expect("catalog should publish before a level exists");

    let manager = runtime
        .handle()
        .resolve_manager::<crate::scene::DefaultLevelManager>(
            crate::scene::DEFAULT_LEVEL_MANAGER_NAME,
        )
        .expect("default level manager should resolve");
    let handle = LevelManager::create_default_level_handle(manager.as_ref())
        .expect("manager contract should create a catalog-aware level");
    let level = manager
        .level(handle)
        .expect("manager-created level should remain registered");

    level.with_world(|world| {
        assert!(world
            .type_registry()
            .contains_type_path("gameplay.Component.FutureHealth"));
    });
}

#[test]
fn catalog_cannot_take_over_a_direct_vm_registration() {
    let direct = vm_component_registration(
        "gameplay.Component.DirectHealth",
        "DirectHealth",
        ReflectScriptVisibility::Public,
        vec![scalar_field("direct")],
    );
    let catalog = vm_component_registration(
        "gameplay.Component.DirectHealth",
        "DirectHealth",
        ReflectScriptVisibility::Public,
        vec![scalar_field("catalog")],
    );
    let mut world = World::empty();
    world
        .register_vm_type(direct, VmTypeBacking::DynamicComponent)
        .expect("direct VM type should register once");

    let error = world
        .sync_vm_types(&[catalog])
        .expect_err("catalog sync must not replace a directly registered type");

    assert!(matches!(
        error,
        SceneError::Reflect(
            zircon_runtime_interface::reflect::ReflectError::DuplicateTypePath { .. }
        )
    ));
    assert_eq!(
        world
            .type_registry()
            .registration("gameplay.Component.DirectHealth")
            .expect("direct registration should remain")
            .type_info
            .fields[0]
            .name,
        "direct"
    );
}

#[test]
fn canonical_registry_rejects_a_collision_in_any_managed_world_before_activation() {
    let catalog = VmReflectionCatalog::default();
    let runtime = reflection_test_runtime(&catalog);
    let first = crate::scene::create_default_level(&runtime.handle())
        .expect("first managed level should be created");
    let second = crate::scene::create_default_level(&runtime.handle())
        .expect("second managed level should be created");
    let direct = vm_component_registration(
        "gameplay.Component.ManagedDirect",
        "ManagedDirect",
        ReflectScriptVisibility::Public,
        vec![scalar_field("direct")],
    );
    second.with_world_mut(|world| {
        world
            .register_vm_type(direct, VmTypeBacking::DynamicComponent)
            .expect("direct registration should install in one managed world");
    });
    first.with_world(|world| {
        assert!(!world
            .type_registry()
            .contains_type_path("gameplay.Component.ManagedDirect"));
    });
    let candidate = vm_component_registration(
        "gameplay.Component.ManagedDirect",
        "ManagedDirect",
        ReflectScriptVisibility::Public,
        vec![scalar_field("catalog")],
    );

    let error = catalog
        .prepare_optional_generation(
            PluginSlotId::new(13),
            1,
            "gameplay",
            Some(&state_schema(vec![candidate])),
        )
        .expect_err("canonical registry must validate every managed world before activation");

    assert!(matches!(
        error,
        VmReflectionError::Reflect(
            zircon_runtime_interface::reflect::ReflectError::DuplicateTypePath { .. }
        ) | VmReflectionError::Scene(SceneError::Reflect(
            zircon_runtime_interface::reflect::ReflectError::DuplicateTypePath { .. }
        ))
    ));
}

#[test]
fn catalog_rejects_builtin_type_path_collision_without_existing_levels() {
    let catalog = VmReflectionCatalog::default();
    let _runtime = reflection_test_runtime(&catalog);
    let mut collision = vm_component_registration(
        "zircon_runtime::scene::components::Name",
        "Name",
        ReflectScriptVisibility::Public,
        vec![scalar_field("value")],
    );
    collision.type_path = collision
        .type_path
        .with_plugin_id("zircon_runtime")
        .expect("test plugin id should be valid");

    let error = catalog
        .publish_generation(
            PluginSlotId::new(12),
            1,
            "zircon_runtime",
            &state_schema(vec![collision]),
        )
        .expect_err("builtin type paths must be reserved before the first level exists");

    assert!(matches!(
        error,
        VmReflectionError::Reflect(
            zircon_runtime_interface::reflect::ReflectError::DuplicateTypePath { .. }
                | zircon_runtime_interface::reflect::ReflectError::InvalidRegistration { .. }
        )
    ));
}

#[test]
fn concurrent_catalog_publications_preserve_every_slot() {
    const SLOT_COUNT: u64 = 16;
    let catalog = Arc::new(VmReflectionCatalog::default());
    let barrier = Arc::new(Barrier::new(SLOT_COUNT as usize));
    let mut workers = Vec::new();
    for slot in 1..=SLOT_COUNT {
        let catalog = Arc::clone(&catalog);
        let barrier = Arc::clone(&barrier);
        workers.push(std::thread::spawn(move || {
            let type_path = format!("gameplay.Component.Concurrent{slot}");
            let short = format!("Concurrent{slot}");
            let schema = state_schema(vec![vm_component_registration(
                &type_path,
                &short,
                ReflectScriptVisibility::Public,
                vec![scalar_field("value")],
            )]);
            barrier.wait();
            catalog
                .publish_generation(PluginSlotId::new(slot), 1, "gameplay", &schema)
                .expect("concurrent distinct slots should publish");
        }));
    }
    for worker in workers {
        worker.join().expect("catalog publisher should not panic");
    }

    let mut world = World::empty();
    catalog
        .apply_to_world(&mut world)
        .expect("final catalog snapshot should apply");
    for slot in 1..=SLOT_COUNT {
        assert!(world
            .type_registry()
            .contains_type_path(&format!("gameplay.Component.Concurrent{slot}")));
    }
}

#[test]
fn initial_dynamic_component_json_must_match_the_registered_schema() {
    let registration = vm_component_registration(
        "gameplay.Component.TypedHealth",
        "TypedHealth",
        ReflectScriptVisibility::Public,
        vec![scalar_field("current")],
    );
    let mut world = World::empty();
    world
        .register_vm_type(registration, VmTypeBacking::DynamicComponent)
        .expect("typed VM component should register");
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");

    let error = world
        .set_dynamic_component(
            entity,
            "gameplay.Component.TypedHealth",
            serde_json::json!({ "current": "not-a-scalar" }),
        )
        .expect_err("initial JSON must not bypass reflected field types");

    assert!(matches!(
        error,
        SceneError::Reflect(zircon_runtime_interface::reflect::ReflectError::TypeMismatch { .. })
    ));
}

#[test]
fn initial_dynamic_component_json_requires_every_registered_field() {
    let registration = vm_component_registration(
        "gameplay.Component.CompleteHealth",
        "CompleteHealth",
        ReflectScriptVisibility::Public,
        vec![scalar_field("current"), scalar_field("maximum")],
    );
    let mut world = World::empty();
    world
        .register_vm_type(registration, VmTypeBacking::DynamicComponent)
        .expect("typed VM component should register");
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");

    let error = world
        .set_dynamic_component(
            entity,
            "gameplay.Component.CompleteHealth",
            serde_json::json!({ "current": 25.0 }),
        )
        .expect_err("initial JSON must contain every compiled reflected field");

    assert!(matches!(
        error,
        SceneError::Reflect(zircon_runtime_interface::reflect::ReflectError::UnknownField {
            field_name,
            ..
        }) if field_name == "maximum"
    ));
}

#[test]
fn initial_dynamic_json_and_dense_reads_share_the_same_declared_value_semantics() {
    let registration = vm_component_registration(
        "gameplay.Component.ValueShapes",
        "ValueShapes",
        ReflectScriptVisibility::Public,
        vec![
            test_field("scalar", "Scalar", ReflectEditorHint::Scalar),
            test_field("unsigned", "Unsigned", ReflectEditorHint::Unsigned),
            test_field("entity", "Entity", ReflectEditorHint::Entity),
            test_field("resource", "Resource", ReflectEditorHint::Resource),
            test_field("rotation", "Quaternion", ReflectEditorHint::Vec4),
            test_field("tags", "List<String>", ReflectEditorHint::Json),
            test_field("weights", "Map<String, Scalar>", ReflectEditorHint::Json),
            test_field("payload", "Json", ReflectEditorHint::Json),
        ],
    );
    let mut world = World::empty();
    world
        .register_vm_type(registration, VmTypeBacking::DynamicComponent)
        .expect("VM value-shape component should register");
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    world
        .set_dynamic_component(
            entity,
            "gameplay.Component.ValueShapes",
            serde_json::json!({
                "scalar": 25,
                "unsigned": 7,
                "entity": { "entity": 42 },
                "resource": { "resource": "mesh://cube" },
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "tags": ["player", "alive"],
                "weights": { "speed": 1.5 },
                "payload": { "arbitrary": true }
            }),
        )
        .expect("all declared VM JSON shapes should validate");
    let address = ReflectObjectAddress::component(entity, "gameplay.Component.ValueShapes")
        .expect("component address should build");

    let cases = [
        ("scalar", ReflectedValue::Scalar(25.0)),
        ("unsigned", ReflectedValue::Unsigned(7)),
        ("entity", ReflectedValue::Entity(Some(42))),
        (
            "resource",
            ReflectedValue::Resource("mesh://cube".to_string()),
        ),
        ("rotation", ReflectedValue::Quaternion([0.0, 0.0, 0.0, 1.0])),
        (
            "tags",
            ReflectedValue::List(vec![
                ReflectedValue::String("player".to_string()),
                ReflectedValue::String("alive".to_string()),
            ]),
        ),
        (
            "weights",
            ReflectedValue::Map(BTreeMap::from([(
                "speed".to_string(),
                ReflectedValue::Scalar(1.5),
            )])),
        ),
        (
            "payload",
            ReflectedValue::Json(serde_json::json!({ "arbitrary": true })),
        ),
    ];
    for (field, expected) in cases {
        let value = world
            .reflect_read(ReflectReadRequest::new(
                address.clone(),
                ReflectFieldId::from_stable_keys("tests.vm-reflection-field", field),
            ))
            .expect("validated initial JSON must remain readable through reflection");
        assert_eq!(value.field.value, expected, "field `{field}` diverged");
    }
}

#[test]
fn canonical_registry_excludes_world_local_direct_vm_schemas() {
    let catalog = VmReflectionCatalog::default();
    let runtime = reflection_test_runtime(&catalog);
    let first = crate::scene::create_default_level(&runtime.handle())
        .expect("first managed level should be created");
    let second = crate::scene::create_default_level(&runtime.handle())
        .expect("second managed level should be created");
    first.with_world_mut(|world| {
        world
            .register_vm_type(
                vm_component_registration(
                    "gameplay.Component.WorldLocal",
                    "WorldLocal",
                    ReflectScriptVisibility::Public,
                    vec![scalar_field("health")],
                ),
                VmTypeBacking::DynamicComponent,
            )
            .expect("first World direct schema should register");
    });
    second.with_world_mut(|world| {
        world
            .register_vm_type(
                vm_component_registration(
                    "gameplay.Component.WorldLocal",
                    "WorldLocal",
                    ReflectScriptVisibility::Public,
                    vec![scalar_field("armor")],
                ),
                VmTypeBacking::DynamicComponent,
            )
            .expect("second World direct schema should register independently");
    });

    let prepared = catalog
        .prepare_optional_generation(PluginSlotId::new(99), 1, "gameplay", None)
        .expect("world-local direct schemas must not alter the process-wide registry");
    let registry = prepared.snapshot().registry();

    assert!(!registry.contains_type_path("gameplay.Component.WorldLocal"));
    assert!(registry.contains_type_path("zircon_runtime::scene::components::Name"));
}

#[test]
fn newer_generation_replaces_same_slot_vm_schema_in_existing_worlds() {
    let catalog = VmReflectionCatalog::default();
    let (_runtime, level) = create_managed_level(&catalog);
    let slot = PluginSlotId::new(7);
    catalog
        .publish_generation(
            slot,
            1,
            "gameplay",
            &state_schema(vec![
                vm_component_registration(
                    "gameplay.Component.Health",
                    "Health",
                    ReflectScriptVisibility::Public,
                    vec![scalar_field("current")],
                ),
                vm_component_registration(
                    "gameplay.Component.Armor",
                    "Armor",
                    ReflectScriptVisibility::Public,
                    vec![scalar_field("current")],
                ),
            ]),
        )
        .expect("first schema generation should publish");

    catalog
        .publish_generation(
            slot,
            2,
            "gameplay",
            &state_schema(vec![vm_component_registration(
                "gameplay.Component.Health",
                "Health",
                ReflectScriptVisibility::Public,
                vec![scalar_field("current"), scalar_field("maximum")],
            )]),
        )
        .expect("newer schema generation should replace the same owned type");

    level.with_world(|world| {
        let registration = world
            .type_registry()
            .registration("gameplay.Component.Health")
            .expect("updated VM type should remain registered");
        assert_eq!(registration.type_info.fields.len(), 2);
        assert_eq!(registration.type_info.fields[1].name, "maximum");
        assert!(!world
            .type_registry()
            .contains_type_path("gameplay.Component.Armor"));
    });
}

#[test]
fn catalog_rejects_removing_a_vm_type_with_live_world_components() {
    let catalog = VmReflectionCatalog::default();
    let (runtime, level) = create_managed_level(&catalog);
    let peer = crate::scene::create_default_level(&runtime.handle())
        .expect("peer managed level should be created before publication");
    let slot = PluginSlotId::new(8);
    catalog
        .publish_generation(
            slot,
            1,
            "gameplay",
            &state_schema(vec![vm_component_registration(
                "gameplay.Component.Health",
                "Health",
                ReflectScriptVisibility::Public,
                vec![scalar_field("current")],
            )]),
        )
        .expect("first schema generation should publish");
    level.with_world_mut(|world| {
        let entity = world
            .spawn_node(NodeKind::Empty)
            .expect("test scene spawn should succeed");
        world
            .set_dynamic_component(
                entity,
                "gameplay.Component.Health",
                serde_json::json!({ "current": 25.0 }),
            )
            .expect("live VM component should attach");
    });

    let error = catalog
        .publish_generation(slot, 2, "gameplay", &state_schema(Vec::new()))
        .expect_err("a live VM component must block schema removal");

    assert!(matches!(
        error,
        VmReflectionError::Scene(SceneError::PluginComponentsActive { .. })
    ));
    level.with_world(|world| {
        assert!(world
            .type_registry()
            .contains_type_path("gameplay.Component.Health"));
    });
    peer.with_world(|world| {
        assert!(world
            .type_registry()
            .contains_type_path("gameplay.Component.Health"));
    });
    let mut future_world = World::empty();
    catalog
        .apply_to_world(&mut future_world)
        .expect("failed candidate must never become visible to future Worlds");
    assert!(future_world
        .type_registry()
        .contains_type_path("gameplay.Component.Health"));
}

#[test]
fn empty_schema_still_rejects_generation_regression() {
    let catalog = VmReflectionCatalog::default();
    let slot = PluginSlotId::new(9);
    catalog
        .publish_optional_generation(slot, 3, "gameplay", None)
        .expect("empty current generation should publish");

    let error = catalog
        .publish_optional_generation(slot, 2, "gameplay", None)
        .expect_err("an empty stale generation must still be rejected");

    assert!(matches!(
        error,
        VmReflectionError::GenerationRegression {
            current_generation: 3,
            requested_generation: 2,
            ..
        }
    ));
}

#[test]
fn discarding_a_slot_removes_its_unused_world_registrations() {
    let catalog = VmReflectionCatalog::default();
    let (_runtime, level) = create_managed_level(&catalog);
    let slot = PluginSlotId::new(10);
    catalog
        .publish_generation(
            slot,
            1,
            "gameplay",
            &state_schema(vec![vm_component_registration(
                "gameplay.Component.Health",
                "Health",
                ReflectScriptVisibility::Public,
                vec![scalar_field("current")],
            )]),
        )
        .expect("schema should publish");

    assert_eq!(
        catalog
            .discard_slot(slot)
            .expect("unused slot registrations should discard"),
        1
    );
    level.with_world(|world| {
        assert!(!world
            .type_registry()
            .contains_type_path("gameplay.Component.Health"));
    });
}

#[test]
fn different_slots_cannot_claim_the_same_vm_type_path() {
    let catalog = VmReflectionCatalog::default();
    let schema = state_schema(vec![vm_component_registration(
        "gameplay.Component.Health",
        "Health",
        ReflectScriptVisibility::Public,
        vec![scalar_field("current")],
    )]);
    catalog
        .publish_generation(PluginSlotId::new(1), 1, "gameplay", &schema)
        .expect("first slot should own the VM type path");

    let error = catalog
        .publish_generation(PluginSlotId::new(2), 1, "gameplay", &schema)
        .expect_err("another slot must not replace an existing VM type owner");

    assert!(matches!(
        error,
        VmReflectionError::TypePathOwnedByAnotherSlot {
            owner_slot,
            requesting_slot,
            ..
        } if owner_slot == PluginSlotId::new(1) && requesting_slot == PluginSlotId::new(2)
    ));
}

fn state_schema(registrations: Vec<ReflectTypeRegistration>) -> VmStateSchema {
    VmStateSchema {
        schema_version: 2,
        types: registrations
            .into_iter()
            .enumerate()
            .map(|(index, registration)| VmStateTypeSchema {
                registration,
                type_hash: index as u32 + 1,
            })
            .collect(),
    }
}

fn vm_component_registration(
    type_path: &str,
    short_type_path: &str,
    visibility: ReflectScriptVisibility,
    fields: Vec<ReflectFieldInfo>,
) -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(type_path, short_type_path)
            .expect("test VM type path should be valid")
            .with_plugin_id("gameplay")
            .expect("test plugin id should be valid"),
        short_type_path,
        ReflectTypeInfo::struct_with_fields(fields),
        ReflectSerializationStrategy::Value,
    )
    .as_component()
    .with_plugin_id("gameplay")
    .expect("test plugin id should be valid")
    .with_script_visibility(visibility)
}

fn scalar_field(name: &str) -> ReflectFieldInfo {
    test_field(name, "Scalar", ReflectEditorHint::Scalar)
}

fn test_field(
    name: &str,
    value_type_path: &str,
    editor_hint: ReflectEditorHint,
) -> ReflectFieldInfo {
    ReflectFieldInfo::from_stable_keys(
        "tests.vm-reflection-field",
        name,
        name,
        value_type_path,
        editor_hint,
    )
}
