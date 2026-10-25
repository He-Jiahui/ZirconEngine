//! B2-U02c-1: package ownership, strict registered paths and Play-only JSON rows.
use serde_json::json;
use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectFieldInfo, ReflectScriptVisibility, ReflectSerializationStrategy,
    ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
};

use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::AssetUri;
use crate::core::framework::scene::{ComponentPropertyPath, ComponentTypeDescriptor};
use crate::scene::{DynamicComponent, DynamicScene, NodeKind, SceneError, World};
use crate::script::{
    PluginSlotId, VmReflectionCatalog, VmReflectionError, VmStateSchema, VmStateTypeSchema,
};

fn plugin_world(owner: &str) -> World {
    let mut world = World::empty();
    world
        .register_component_type(ComponentTypeDescriptor::new(
            format!("{owner}.Tag"),
            owner,
            "Tag",
        ))
        .unwrap();
    world
}

fn load(catalog: &VmReflectionCatalog, slot: u64, generation: u32, owner: &str) {
    catalog
        .publish_optional_generation(PluginSlotId::new(slot), generation, owner, None)
        .expect("a schema-less package should own its namespace");
}

fn orb_world() -> (VmReflectionCatalog, World, crate::scene::EntityId) {
    let catalog = VmReflectionCatalog::default();
    load(&catalog, 1, 1, "orb_run");
    let mut world = plugin_world("rendering");
    catalog.apply_to_world(&mut world).unwrap();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    (catalog, world, entity)
}

fn bindings() -> serde_json::Value {
    json!([{
        "package": "orb_run", "module": "main", "enabled": true,
        "update": true, "fixed_update": false, "properties": {}
    }])
}

fn admission_snapshot(world: &World) -> serde_json::Value {
    // World::clone rebuilds derived bookkeeping. Compare the admission contract itself,
    // including unsaved rows and catalog/generation state, rather than those caches.
    json!({
        "persisted": serde_json::to_value(world).unwrap(),
        "namespaces": &world.script_namespaces,
        "rows": &world.dynamic_components,
        "row_generations": &world.dynamic_component_generations,
        "descriptors": world.component_types.descriptors().collect::<Vec<_>>(),
        "types": world.type_registry.iter().map(|entry| &entry.registration).collect::<Vec<_>>(),
        "vm_catalog": &world.vm_catalog_type_paths,
        "vm_dynamic": &world.vm_dynamic_type_paths,
        "world_generation": world.world_generation(),
        "component_generation": world.component_registry_generation(),
        "change_tick": world.read_change_tick().get(),
    })
}

fn typed_state() -> VmStateSchema {
    let registration = ReflectTypeRegistration::new(
        ReflectTypePath::new("orb_run.Health", "Health")
            .unwrap()
            .with_plugin_id("orb_run")
            .unwrap(),
        "Health",
        ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::from_stable_keys(
            "tests.script-namespace",
            "hp",
            "hp",
            "Scalar",
            ReflectEditorHint::Scalar,
        )]),
        ReflectSerializationStrategy::Value,
    )
    .as_component()
    .with_plugin_id("orb_run")
    .unwrap()
    .with_script_visibility(ReflectScriptVisibility::Public);
    VmStateSchema {
        schema_version: 2,
        types: vec![VmStateTypeSchema {
            registration,
            type_hash: 1,
        }],
    }
}

fn managed_runtime(catalog: &VmReflectionCatalog) -> crate::core::CoreRuntime {
    use crate::core::{CoreRuntime, TasksModule, TimeModule};
    use crate::engine_module::EngineModule;
    let runtime = CoreRuntime::new();
    let mut names = Vec::new();
    for descriptor in [
        TimeModule.descriptor(),
        crate::foundation::module_descriptor(),
        TasksModule.descriptor(),
        crate::asset::module_descriptor(),
        crate::scene::module_descriptor(),
    ] {
        names.push(descriptor.name.clone());
        runtime.register_module(descriptor).unwrap();
    }
    for name in names {
        runtime.activate_module(&name).unwrap();
    }
    catalog.bind_core(&runtime.handle());
    crate::scene::install_world_runtime_extension_plan(
        &runtime.handle(),
        catalog.world_runtime_extension_plan().unwrap(),
    )
    .unwrap();
    runtime
}

#[test]
fn package_lifecycle_syncs_existing_and_future_managed_worlds() {
    let catalog = VmReflectionCatalog::default();
    let runtime = managed_runtime(&catalog);
    let existing = crate::scene::create_default_level(&runtime.handle()).unwrap();
    existing.with_world_mut(|world| {
        world
            .register_component_type(ComponentTypeDescriptor::new(
                "rendering.Tag",
                "rendering",
                "Tag",
            ))
            .unwrap();
    });
    load(&catalog, 1, 1, "orb_run");
    let future = crate::scene::create_default_level(&runtime.handle()).unwrap();
    for level in [&existing, &future] {
        level.with_world_mut(|world| {
            let entity = world.spawn_node(NodeKind::Empty).unwrap();
            world
                .set_dynamic_component(entity, "orb_run.run_state", json!("playing"))
                .unwrap();
        });
    }
    catalog.discard_slot(PluginSlotId::new(1)).unwrap();
    for level in [&existing, &future] {
        level.with_world(|world| {
            for node in world.node_records() {
                assert!(world
                    .dynamic_component(node.id, "orb_run.run_state")
                    .is_none());
            }
        });
    }
}

#[test]
fn native_collision_in_one_managed_world_rejects_all_publication() {
    let catalog = VmReflectionCatalog::default();
    let runtime = managed_runtime(&catalog);
    let first = crate::scene::create_default_level(&runtime.handle()).unwrap();
    let second = crate::scene::create_default_level(&runtime.handle()).unwrap();
    second.with_world_mut(|world| {
        world
            .register_component_type(ComponentTypeDescriptor::new(
                "orb_run.Native",
                "orb_run",
                "Native",
            ))
            .unwrap();
    });
    let first_before = first.with_world(admission_snapshot);
    let second_before = second.with_world(admission_snapshot);
    assert!(catalog
        .publish_optional_generation(PluginSlotId::new(1), 1, "orb_run", None)
        .is_err());
    assert_eq!(catalog.revision(), 0);
    first.with_world(|world| assert_eq!(admission_snapshot(world), first_before));
    second.with_world(|world| assert_eq!(admission_snapshot(world), second_before));
}

#[test]
fn schema_less_package_admits_json_with_unrelated_native_descriptors() {
    let (_catalog, mut world, entity) = orb_world();
    for value in [
        json!("playing"),
        json!(7),
        json!([1, 2]),
        json!({"score": 8}),
    ] {
        world
            .set_dynamic_component(entity, "orb_run.run_state", value.clone())
            .unwrap();
        assert_eq!(
            world.dynamic_component(entity, "orb_run.run_state"),
            Some(&value)
        );
    }
    for id in [
        "gameplay.menu_state",
        "gameplay.control_state",
        "gameplay.hud_text",
    ] {
        world
            .set_dynamic_component(entity, id, json!("runtime state"))
            .unwrap();
    }
    assert!(world
        .component_type_descriptor("orb_run.run_state")
        .is_none());
}

#[test]
fn namespace_requires_loaded_package_and_is_world_local() {
    let (_catalog, _loaded, _) = orb_world();
    let mut unrelated = plugin_world("rendering");
    let entity = unrelated.spawn_node(NodeKind::Empty).unwrap();
    unrelated
        .set_dynamic_component(entity, "script.bindings", bindings())
        .unwrap();
    assert!(matches!(
        unrelated.set_dynamic_component(entity, "orb_run.run_state", json!("playing")),
        Err(SceneError::UnregisteredDynamicComponentType { .. })
    ));
}

#[test]
fn rejected_paths_name_the_allowed_package_prefix() {
    let (_catalog, mut world, entity) = orb_world();
    for id in [
        "orb.run_state",
        "orb_run_extra.state",
        "orb_run",
        "orb_run.",
        "unknown.State",
        "gameplay.custom_state",
    ] {
        let error = world
            .set_dynamic_component(entity, id, json!("bad"))
            .expect_err(id);
        assert!(
            matches!(&error, SceneError::ScriptComponentNamespaceViolation { allowed_prefixes, .. }
            if allowed_prefixes == "orb_run.")
        );
        assert!(error.to_string().contains("orb_run."), "{error}");
        assert!(world.dynamic_component(entity, id).is_none());
    }
}

#[test]
fn untyped_property_writes_use_the_same_namespace_gate() {
    let (_catalog, mut world, entity) = orb_world();
    let path = ComponentPropertyPath::new("orb_run.stats", vec!["score".to_string()]).unwrap();
    world
        .set_dynamic_component_json_property(entity, &path, json!(8))
        .unwrap();
    assert_eq!(
        world.dynamic_component(entity, "orb_run.stats"),
        Some(&json!({"score":8}))
    );
    let foreign = ComponentPropertyPath::new("orb.stats", vec!["score".to_string()]).unwrap();
    let error = world
        .set_dynamic_component_json_property(entity, &foreign, json!(8))
        .unwrap_err();
    assert!(error.to_string().contains("orb_run."));
}

#[test]
fn exact_registered_path_keeps_strict_value_and_property_checks() {
    let (catalog, mut world, entity) = orb_world();
    catalog
        .publish_generation(PluginSlotId::new(1), 2, "orb_run", &typed_state())
        .unwrap();
    catalog.apply_to_world(&mut world).unwrap();
    assert!(matches!(
        world.set_dynamic_component(entity, "orb_run.Health", json!("untyped")),
        Err(SceneError::DynamicComponentNotObject { .. })
    ));
    for value in [
        json!({}),
        json!({"hp":"wrong"}),
        json!({"hp":1,"extra":true}),
    ] {
        assert!(world
            .set_dynamic_component(entity, "orb_run.Health", value)
            .is_err());
    }
    world
        .set_dynamic_component(entity, "orb_run.Health", json!({"hp":5}))
        .unwrap();
    let extra = ComponentPropertyPath::new("orb_run.Health", vec!["extra".to_string()]).unwrap();
    assert!(world
        .set_dynamic_component_json_property(entity, &extra, json!(1))
        .is_err());
    assert_eq!(
        world.dynamic_component(entity, "orb_run.Health"),
        Some(&json!({"hp":5}))
    );
    world
        .set_dynamic_component(entity, "orb_run.other", json!("still untyped"))
        .unwrap();
}

#[test]
fn duplicate_schema_less_owners_are_rejected_but_same_slot_reload_is_allowed() {
    let catalog = VmReflectionCatalog::default();
    load(&catalog, 1, 1, "orb_run");
    let revision = catalog.revision();
    let error = catalog
        .publish_optional_generation(PluginSlotId::new(2), 1, "orb_run", None)
        .expect_err("two loaded packages cannot share a namespace");
    assert!(
        matches!(&error, VmReflectionError::NamespaceOwnedByAnotherSlot { owner_slot, requesting_slot, .. }
        if *owner_slot == PluginSlotId::new(1) && *requesting_slot == PluginSlotId::new(2))
    );
    assert!(error.to_string().contains("orb_run"));
    assert_eq!(catalog.revision(), revision);
    load(&catalog, 1, 2, "orb_run");
    catalog.discard_slot(PluginSlotId::new(1)).unwrap();
    load(&catalog, 2, 1, "orb_run");
}

#[test]
fn invalid_or_reserved_package_names_do_not_publish() {
    for owner in [
        "", "orb.run", " orb_run", "orb_run ", "script", "zircon", "gameplay", "runtime", "editor",
        "zr",
    ] {
        let catalog = VmReflectionCatalog::default();
        let error = catalog
            .publish_optional_generation(PluginSlotId::new(1), 1, owner, None)
            .expect_err(owner);
        assert!(
            matches!(&error, VmReflectionError::InvalidPackageNamespace { package_name, .. }
            if package_name == owner)
        );
        assert!(error.to_string().contains("package"), "{error}");
        assert_eq!(catalog.revision(), 0);
    }
}

#[test]
fn native_plugin_namespace_collision_does_not_publish_to_world() {
    let catalog = VmReflectionCatalog::default();
    load(&catalog, 1, 1, "orb_run");
    let mut world = plugin_world("orb_run");
    let before = admission_snapshot(&world);
    let error = catalog
        .apply_to_world(&mut world)
        .expect_err("native plugin ids are reserved");
    assert!(
        matches!(&error, VmReflectionError::Scene(SceneError::ScriptNamespaceNativePluginConflict { package_name })
        if package_name == "orb_run")
    );
    assert!(error.to_string().contains("orb_run"));
    assert_eq!(admission_snapshot(&world), before);
}

#[test]
fn native_plugin_cannot_take_an_already_loaded_package_namespace() {
    let (_catalog, mut world, _) = orb_world();
    let error = world
        .register_component_type(ComponentTypeDescriptor::new(
            "orb_run.Native",
            "orb_run",
            "Native",
        ))
        .expect_err("native registration cannot take an owned namespace");
    assert!(error.to_string().contains("orb_run"));
    assert!(world.component_type_descriptor("orb_run.Native").is_none());
}

#[test]
fn hot_reload_retains_untyped_state_and_unload_discards_it() {
    let (catalog, mut world, entity) = orb_world();
    world
        .set_dynamic_component(entity, "orb_run.run_state", json!("playing"))
        .unwrap();
    load(&catalog, 1, 2, "orb_run");
    catalog.apply_to_world(&mut world).unwrap();
    assert_eq!(
        world.dynamic_component(entity, "orb_run.run_state"),
        Some(&json!("playing"))
    );
    catalog.discard_slot(PluginSlotId::new(1)).unwrap();
    catalog.apply_to_world(&mut world).unwrap();
    assert!(world
        .dynamic_component(entity, "orb_run.run_state")
        .is_none());
    assert!(world
        .set_dynamic_component(entity, "orb_run.run_state", json!("stale"))
        .is_err());
}

#[test]
fn clones_retain_play_state_but_serde_saves_omit_untyped_rows_and_namespace() {
    let (_catalog, mut world, entity) = orb_world();
    world
        .set_dynamic_component(entity, "script.bindings", bindings())
        .unwrap();
    world
        .set_dynamic_component(entity, "orb_run.run_state", json!("playing"))
        .unwrap();
    let mut clone = world.clone();
    clone
        .set_dynamic_component(entity, "orb_run.score", json!(3))
        .unwrap();
    assert_eq!(
        clone.dynamic_component(entity, "orb_run.run_state"),
        Some(&json!("playing"))
    );
    let saved = serde_json::to_string(&world).unwrap();
    assert!(!saved.contains("orb_run.run_state"));
    let mut reopened: World = serde_json::from_str(&saved).unwrap();
    assert!(reopened
        .dynamic_component(entity, "script.bindings")
        .is_some());
    reopened
        .register_component_type(ComponentTypeDescriptor::new(
            "rendering.Tag",
            "rendering",
            "Tag",
        ))
        .unwrap();
    assert!(reopened
        .set_dynamic_component(entity, "orb_run.run_state", json!("stale"))
        .is_err());
}

#[test]
fn scene_asset_save_skips_play_rows_and_preserves_bindings() {
    // Validation may retain the fixture in its caller-owned evidence directory.
    let root = std::env::var_os("ZE_SCRIPT_NAMESPACE_TEST_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(format!("project-{}", uuid::Uuid::new_v4()));
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();
    ProjectManifest::new(
        "Namespace",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let project = ProjectManager::open(&root).unwrap();
    let (_catalog, mut world, entity) = orb_world();
    world
        .set_dynamic_component(entity, "script.bindings", bindings())
        .unwrap();
    world
        .set_dynamic_component(entity, "orb_run.run_state", json!("playing"))
        .unwrap();
    let scene = world
        .to_scene_asset(&project)
        .expect("Play rows must neither persist nor block saves");
    assert!(!serde_json::to_string(&scene)
        .unwrap()
        .contains("orb_run.run_state"));
    let reopened = World::from_scene_asset(&project, &scene).unwrap();
    assert!(reopened
        .dynamic_component(entity, "script.bindings")
        .is_some());
    assert!(reopened
        .dynamic_component(entity, "orb_run.run_state")
        .is_none());
}

#[test]
fn dynamic_spawn_uses_shared_admission_for_builtin_paths() {
    let (_catalog, mut target, _) = orb_world();
    let mut source = World::empty();
    source.spawn_node(NodeKind::Empty).unwrap();
    for id in ["script.bindings", "gameplay.menu_state"] {
        let mut scene = DynamicScene::from_world(&source).unwrap();
        scene.entities[0]
            .components
            .push(DynamicComponent::new(id, true, Vec::new()));
        let remap = scene
            .spawn_into(&mut target)
            .expect("spawn preflight must share World admission");
        let entity = remap.get(scene.entities[0].source_entity).unwrap();
        assert_eq!(target.dynamic_component(entity, id), Some(&json!({})));
    }
}

#[test]
fn dynamic_preflight_retains_package_namespace_context() {
    let (_catalog, target, _) = orb_world();
    let mut preflight = target.dynamic_scene_preflight_world(["rendering.Tag", "orb_run.stats"]);
    let entity = preflight.spawn_node(NodeKind::Empty).unwrap();
    preflight
        .set_dynamic_component(entity, "orb_run.stats", json!({"score":8}))
        .unwrap();
}

#[test]
fn empty_catalog_compatibility_fallback_remains_available() {
    let mut world = World::empty();
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world
        .set_dynamic_component(entity, "legacy.arbitrary", json!("allowed"))
        .unwrap();
    assert_eq!(
        world.dynamic_component(entity, "legacy.arbitrary"),
        Some(&json!("allowed"))
    );
    let catalog = VmReflectionCatalog::default();
    load(&catalog, 1, 1, "orb_run");
    catalog.apply_to_world(&mut world).unwrap();
    world
        .set_dynamic_component(entity, "legacy.another", json!("still allowed"))
        .unwrap();
}

#[test]
fn legacy_gate_keeps_builtin_aliases_and_any_id_with_empty_dynamic_catalog() {
    let world = plugin_world("rendering");
    assert!(world.admits_dynamic_component_type("Name"));

    let mut empty = World::empty();
    empty
        .type_registry
        .register(crate::scene::RuntimeTypeRegistration::metadata(
            ReflectTypeRegistration::new(
                ReflectTypePath::new("compat.Resource", "Resource").unwrap(),
                "Compatibility resource",
                ReflectTypeInfo::json_with_fields(Vec::new()),
                ReflectSerializationStrategy::Json,
            )
            .as_resource(),
        ))
        .unwrap();
    assert!(empty.admits_dynamic_component_type("compat.Resource"));
}

#[test]
fn namespace_changes_invalidate_prepared_world_mutations_even_without_rows() {
    let (catalog, mut world, _) = orb_world();
    let before = world.world_generation();
    catalog.discard_slot(PluginSlotId::new(1)).unwrap();
    catalog.apply_to_world(&mut world).unwrap();
    assert_ne!(world.world_generation(), before);
}

#[test]
fn typed_upgrade_rejects_incompatible_untyped_state_without_partial_publication() {
    let (catalog, mut world, entity) = orb_world();
    world
        .set_dynamic_component(entity, "orb_run.Health", json!("bare string"))
        .unwrap();
    catalog
        .publish_generation(PluginSlotId::new(1), 2, "orb_run", &typed_state())
        .unwrap();
    let before = admission_snapshot(&world);
    assert!(catalog.apply_to_world(&mut world).is_err());
    assert_eq!(admission_snapshot(&world), before);
    world
        .set_dynamic_component(entity, "orb_run.other", json!("namespace retained"))
        .unwrap();
}
