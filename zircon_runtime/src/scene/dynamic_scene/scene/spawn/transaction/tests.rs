use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use zircon_runtime_interface::reflect::ReflectError;

#[cfg(feature = "profiling")]
use crate::core::diagnostics::profiling::{
    ProfileCaptureConfig, reset_capture, snapshot, start_capture, test_capture_lock,
};
use crate::scene::{
    DynamicComponent, NodeKind, World, dynamic_scene::PreparedDynamicSceneSpawn, ecs::Component,
    reflect::VmTypeBacking,
};

use super::{
    DynamicScene, apply_compiled_scene_spawn, capture_compiled_scene_spawn_preflight,
    commit_preflighted_compiled_scene_spawn, compile_scene_spawn,
    validate_compiled_scene_spawn_preflight,
};
#[path = "tests/resources.rs"]
mod resources;

use resources::{
    RejectingResource, SlotResource, SlotResourceWriteBudgetReset, register_rejecting_resource,
    register_slot_resource,
};

#[test]
fn compiled_spawn_applies_the_previewed_entity_remap() {
    let mut source = World::empty();
    let source_entity = source.spawn_node(NodeKind::Empty);
    let scene = DynamicScene::from_world(&source).expect("source world should capture");

    let mut target = World::empty();
    target.spawn_node(NodeKind::Cube);
    let plan = compile_scene_spawn(&scene, &target).expect("scene should compile for target");
    assert_eq!(plan.preview.entity_remaps.len(), 1);
    let planned_target = plan.preview.entity_remaps[0].target_entity;
    assert_ne!(planned_target, source_entity);

    let remap = apply_compiled_scene_spawn(&mut target, plan).expect("compiled scene should apply");
    assert_eq!(remap.get(source_entity), Some(planned_target));
    assert!(target.contains_entity(planned_target));
}

#[test]
fn compiled_spawn_publishes_one_final_state_for_small_and_medium_scene_sizes() {
    for entity_count in [1, 1_000] {
        assert_compiled_spawn_final_publication(entity_count);
    }
}

#[test]
#[ignore = "managed performance probe: run under the Runtime08 profiling validation gate"]
fn compiled_spawn_publishes_one_final_state_for_one_hundred_thousand_entities() {
    assert_compiled_spawn_final_publication(100_000);
}

fn assert_compiled_spawn_final_publication(entity_count: usize) {
    let mut source = World::empty();
    for _ in 0..entity_count {
        source.spawn_node(NodeKind::Empty);
    }
    let scene = DynamicScene::from_world(&source).expect("source world should capture");

    let mut target = World::empty();
    let name_adds = Arc::new(AtomicUsize::new(0));
    let observed_name_adds = Arc::clone(&name_adds);
    target.observe_component_lifecycle::<crate::scene::components::Name>(
        crate::scene::ecs::LifecycleEventKind::Add,
        move |_world, _event| {
            observed_name_adds.fetch_add(1, Ordering::Relaxed);
        },
    );
    let generation_before = target.world_generation();

    let remap = scene
        .spawn_into(&mut target)
        .expect("compiled scene should publish every staged entity once");

    assert_eq!(remap.len(), entity_count);
    assert_eq!(target.node_records().len(), entity_count);
    assert_eq!(
        target.world_generation(),
        generation_before.saturating_add(1),
        "the compiled mutation must publish the target world once"
    );
    assert_eq!(
        name_adds.load(Ordering::Relaxed),
        entity_count,
        "each final entity must publish its Name once without intermediate rows"
    );
}

#[test]
fn compiled_spawn_rejects_a_target_generation_change_before_apply() {
    let mut source = World::empty();
    source.spawn_node(NodeKind::Empty);
    let scene = DynamicScene::from_world(&source).expect("source world should capture");

    let mut target = World::empty();
    let expected_generation = target.world_generation();
    let plan = compile_scene_spawn(&scene, &target).expect("scene should compile for target");

    target.spawn_node(NodeKind::Cube);
    let actual_generation = target.world_generation();
    let error = apply_compiled_scene_spawn(&mut target, plan)
        .expect_err("a stale compiled spawn plan must not mutate the target");

    assert_eq!(
        error,
        crate::scene::dynamic_scene::DynamicSceneError::TargetWorldChanged {
            expected_generation,
            actual_generation,
        }
    );
    assert_eq!(target.node_records().len(), 1);
}

#[test]
fn compiled_spawn_rejects_a_component_schema_catalog_change_before_apply() {
    let mut source = World::empty();
    source.spawn_node(NodeKind::Empty);
    let scene = DynamicScene::from_world(&source).expect("source world should capture");

    let mut target = World::empty();
    let expected_generation = target.type_registry().schema_catalog_generation();
    let plan = compile_scene_spawn(&scene, &target).expect("scene should compile for target");
    target
        .register_component_type(crate::core::framework::scene::ComponentTypeDescriptor::new(
            "tests.Component.Marker",
            "tests",
            "Marker",
        ))
        .expect("test component type should register");
    let actual_generation = target.type_registry().schema_catalog_generation();

    let error = apply_compiled_scene_spawn(&mut target, plan)
        .expect_err("a schema-stale compiled spawn plan must not mutate the target");

    assert_eq!(
        error,
        crate::scene::dynamic_scene::DynamicSceneError::TargetSchemaChanged {
            expected_generation,
            actual_generation,
        }
    );
    assert!(target.node_records().is_empty());
}

#[test]
fn compiled_spawn_commit_rejects_a_resource_change_after_preflight() {
    let mut source = World::empty();
    register_slot_resource(&mut source);
    source.spawn_node(NodeKind::Empty);
    source.insert_resource(SlotResource { value: 17 });
    let scene = DynamicScene::from_world(&source).expect("source scene should capture");

    let mut target = World::empty();
    register_slot_resource(&mut target);
    target.insert_resource(SlotResource { value: 3 });
    let plan = compile_scene_spawn(&scene, &target).expect("scene should compile for target");
    let expected_tick = target.read_change_tick().get();
    let (mut preflight, _) = capture_compiled_scene_spawn_preflight(&target, &plan, usize::MAX)
        .expect("preflight target should stage");
    let mutation = validate_compiled_scene_spawn_preflight(&mut preflight, plan)
        .expect("preflight writes should materialize a commit artifact");

    *target
        .get_resource_mut::<SlotResource>()
        .expect("target resource should remain mutable") = SlotResource { value: 29 };
    let actual_tick = target.read_change_tick().get();
    assert_ne!(actual_tick, expected_tick);

    let error = commit_preflighted_compiled_scene_spawn(&mut target, mutation)
        .expect_err("a post-preflight resource mutation must reject the stale commit artifact");
    assert_eq!(
        error,
        crate::scene::dynamic_scene::DynamicSceneError::TargetChangeTickChanged {
            expected_tick,
            actual_tick,
        }
    );
    assert_eq!(
        target.get_resource::<SlotResource>(),
        Some(&SlotResource { value: 29 })
    );
    assert!(target.node_records().is_empty());
}

#[test]
fn compiled_spawn_commit_rejects_a_resource_removal_after_preflight() {
    let mut source = World::empty();
    register_slot_resource(&mut source);
    source.spawn_node(NodeKind::Empty);
    source.insert_resource(SlotResource { value: 17 });
    let scene = DynamicScene::from_world(&source).expect("source scene should capture");

    let mut target = World::empty();
    register_slot_resource(&mut target);
    target.insert_resource(SlotResource { value: 3 });
    let plan = compile_scene_spawn(&scene, &target).expect("scene should compile for target");
    let expected_tick = target.read_change_tick().get();
    let (mut preflight, _) = capture_compiled_scene_spawn_preflight(&target, &plan, usize::MAX)
        .expect("preflight target should stage");
    let mutation = validate_compiled_scene_spawn_preflight(&mut preflight, plan)
        .expect("preflight writes should materialize a commit artifact");

    assert_eq!(
        target.remove_resource::<SlotResource>(),
        Some(SlotResource { value: 3 })
    );
    let actual_tick = target.read_change_tick().get();
    assert_ne!(actual_tick, expected_tick);

    let error = commit_preflighted_compiled_scene_spawn(&mut target, mutation)
        .expect_err("a post-preflight resource removal must reject the stale commit artifact");
    assert_eq!(
        error,
        crate::scene::dynamic_scene::DynamicSceneError::TargetChangeTickChanged {
            expected_tick,
            actual_tick,
        }
    );
    assert!(target.get_resource::<SlotResource>().is_none());
    assert!(target.node_records().is_empty());
}

#[test]
fn compiled_spawn_commit_rejects_a_component_registry_change_after_preflight() {
    let mut source = World::empty();
    source.spawn_node(NodeKind::Empty);
    let scene = DynamicScene::from_world(&source).expect("source scene should capture");

    let mut target = World::empty();
    let plan = compile_scene_spawn(&scene, &target).expect("scene should compile for target");
    let (mut preflight, _) = capture_compiled_scene_spawn_preflight(&target, &plan, usize::MAX)
        .expect("preflight target should stage");
    let mutation = validate_compiled_scene_spawn_preflight(&mut preflight, plan)
        .expect("preflight rows should materialize a commit artifact");

    let expected_generation = target.component_registry_generation();
    target.component_id::<TargetOnlyComponent>();
    let actual_generation = target.component_registry_generation();
    assert_ne!(actual_generation, expected_generation);

    let error = commit_preflighted_compiled_scene_spawn(&mut target, mutation)
        .expect_err("a component registry mutation must reject the stale row artifact");
    assert_eq!(
        error,
        crate::scene::dynamic_scene::DynamicSceneError::TargetComponentRegistryChanged {
            expected_generation,
            actual_generation,
        }
    );
    assert!(target.node_records().is_empty());
}

#[test]
fn staged_compiled_spawn_rejects_a_resource_removal_before_commit() {
    let mut source = World::empty();
    register_slot_resource(&mut source);
    source.spawn_node(NodeKind::Empty);
    source.insert_resource(SlotResource { value: 17 });
    let prepared = PreparedDynamicSceneSpawn::new(
        DynamicScene::from_world(&source).expect("source scene should capture"),
    )
    .expect("source scene should prepare");

    let mut target = World::empty();
    register_slot_resource(&mut target);
    target.insert_resource(SlotResource { value: 3 });
    let staged = prepared
        .stage_into(&mut target)
        .expect("prepared scene should stage against the target snapshot");
    let expected_tick = target.read_change_tick().get();

    assert_eq!(
        target.remove_resource::<SlotResource>(),
        Some(SlotResource { value: 3 })
    );
    let actual_tick = target.read_change_tick().get();
    let error = staged
        .commit_into(&mut target)
        .expect_err("a removal after staging must reject the stale commit artifact");

    assert_eq!(
        error,
        crate::scene::dynamic_scene::DynamicSceneError::TargetChangeTickChanged {
            expected_tick,
            actual_tick,
        }
    );
    assert!(target.get_resource::<SlotResource>().is_none());
    assert!(target.node_records().is_empty());
}

#[test]
fn prepared_commit_uses_the_preflighted_component_descriptors() {
    let descriptor = crate::core::framework::scene::ComponentTypeDescriptor::new(
        "tests.PreparedCommitMarker",
        "tests",
        "PreparedCommitMarker",
    );
    let mut scene = DynamicScene::empty();
    scene.component_types.push(descriptor.clone());
    let mut target = World::empty();
    let plan = compile_scene_spawn(&scene, &target).expect("scene should compile for target");
    let (mut preflight, _) = capture_compiled_scene_spawn_preflight(&target, &plan, usize::MAX)
        .expect("preflight target should stage");
    let mutation = validate_compiled_scene_spawn_preflight(&mut preflight, plan)
        .expect("preflight descriptors should materialize a commit artifact");

    scene.component_types[0] = crate::core::framework::scene::ComponentTypeDescriptor::new(
        "tests.PreparedCommitMarker",
        "tests",
        "LaterCallerScene",
    );
    commit_preflighted_compiled_scene_spawn(&mut target, mutation)
        .expect("commit must use its owned preflighted descriptors, not a caller scene");
    assert_eq!(
        target.component_type_descriptor(&descriptor.type_id),
        Some(&descriptor)
    );
}

#[test]
fn prepared_commit_keeps_plugin_rows_bound_to_its_preflighted_descriptor() {
    const TYPE_ID: &str = "tests.PreparedCommitPluginMarker";
    let descriptor = crate::core::framework::scene::ComponentTypeDescriptor::new(
        TYPE_ID,
        "tests",
        "Prepared Commit Plugin Marker",
    );
    let mut source = World::empty();
    let source_entity = source.spawn_node(NodeKind::Empty);
    let mut scene = DynamicScene::from_world(&source).expect("source scene should capture");
    scene.component_types.push(descriptor.clone());
    let entity = scene
        .entities
        .iter_mut()
        .find(|entity| entity.source_entity == source_entity)
        .expect("captured scene should retain the source entity");
    entity
        .components
        .push(crate::scene::dynamic_scene::DynamicComponent::new(
            TYPE_ID,
            true,
            Vec::new(),
        ));

    let mut target = World::empty();
    let plan = compile_scene_spawn(&scene, &target).expect("scene should compile for target");
    let (mut preflight, _) = capture_compiled_scene_spawn_preflight(&target, &plan, usize::MAX)
        .expect("preflight target should stage");
    let mutation = validate_compiled_scene_spawn_preflight(&mut preflight, plan)
        .expect("preflight plugin row should materialize a commit artifact");

    scene.component_types[0] = crate::core::framework::scene::ComponentTypeDescriptor::new(
        TYPE_ID,
        "tests",
        "Later Caller Plugin Descriptor",
    );
    let remap = commit_preflighted_compiled_scene_spawn(&mut target, mutation)
        .expect("commit must publish the artifact-owned descriptor and plugin row");
    let target_entity = remap
        .get(source_entity)
        .expect("commit should return the artifact-owned entity remap");
    assert_eq!(target.component_type_descriptor(TYPE_ID), Some(&descriptor));
    assert_eq!(
        target.dynamic_component(target_entity, TYPE_ID),
        Some(&serde_json::json!({}))
    );
}

#[test]
fn compiled_spawn_projects_only_affected_target_schema_into_preflight() {
    const SELECTED_TYPE_ID: &str = "tests.TargetRegisteredVmComponent";
    const SELECTED_SHORT_TYPE_PATH: &str = "TargetRegisteredVmComponent";
    const UNSELECTED_TYPE_ID: &str = "tests.UnselectedTargetComponent";
    let selected_descriptor = crate::core::framework::scene::ComponentTypeDescriptor::new(
        SELECTED_TYPE_ID,
        "tests",
        "Target Registered VM Component",
    );
    let unselected_descriptor = crate::core::framework::scene::ComponentTypeDescriptor::new(
        UNSELECTED_TYPE_ID,
        "tests",
        "Unselected Target Component",
    );

    let mut source = World::empty();
    let source_entity = source.spawn_node(NodeKind::Empty);
    let mut scene = DynamicScene::from_world(&source).expect("source scene should capture");
    assert!(
        scene
            .component_types
            .iter()
            .all(|descriptor| descriptor.type_id != SELECTED_TYPE_ID)
    );
    scene
        .entities
        .iter_mut()
        .find(|entity| entity.source_entity == source_entity)
        .expect("captured scene should retain the source entity")
        .components
        .push(DynamicComponent::new(
            SELECTED_SHORT_TYPE_PATH,
            true,
            Vec::new(),
        ));

    let mut target = World::empty();
    target
        .register_vm_type(
            crate::scene::reflect::registration_from_component_descriptor(&selected_descriptor)
                .expect("selected VM registration must derive from its descriptor"),
            VmTypeBacking::DynamicComponent,
        )
        .expect("selected target VM schema should register");
    target
        .register_component_type(unselected_descriptor)
        .expect("unselected target schema should register");

    let plan = compile_scene_spawn(&scene, &target)
        .expect("target-registered plugin component should compile without a scene descriptor");
    let (mut preflight, _) = capture_compiled_scene_spawn_preflight(&target, &plan, usize::MAX)
        .expect("affected target schema should project into preflight");
    assert!(
        preflight
            .component_type_descriptor(SELECTED_TYPE_ID)
            .is_some()
    );
    assert!(preflight.type_registry().contains(SELECTED_TYPE_ID));
    assert!(preflight.is_vm_dynamic_type_path(SELECTED_TYPE_ID));
    assert!(
        preflight
            .component_type_descriptor(UNSELECTED_TYPE_ID)
            .is_none()
    );
    assert!(!preflight.type_registry().contains(UNSELECTED_TYPE_ID));

    let mutation = validate_compiled_scene_spawn_preflight(&mut preflight, plan)
        .expect("projected VM schema should validate the compiled plugin write");
    let remap = commit_preflighted_compiled_scene_spawn(&mut target, mutation)
        .expect("validated affected-schema artifact should publish");
    let target_entity = remap
        .get(source_entity)
        .expect("commit should preserve the planned entity remap");
    assert_eq!(
        target.dynamic_component(target_entity, SELECTED_TYPE_ID),
        Some(&serde_json::json!({}))
    );
}

#[test]
fn compiled_spawn_rejects_unknown_plugin_type_when_target_catalog_is_nonempty() {
    const KNOWN_TYPE_ID: &str = "tests.KnownTargetComponent";
    const UNKNOWN_TYPE_ID: &str = "tests.UnknownSceneComponent";
    let mut source = World::empty();
    let source_entity = source.spawn_node(NodeKind::Empty);
    let mut scene = DynamicScene::from_world(&source).expect("source scene should capture");
    scene
        .entities
        .iter_mut()
        .find(|entity| entity.source_entity == source_entity)
        .expect("captured scene should retain the source entity")
        .components
        .push(DynamicComponent::new(UNKNOWN_TYPE_ID, true, Vec::new()));

    let mut target = World::empty();
    target
        .register_component_type(crate::core::framework::scene::ComponentTypeDescriptor::new(
            KNOWN_TYPE_ID,
            "tests",
            "Known Target Component",
        ))
        .expect("known target component should make the catalog strict");
    let plan = compile_scene_spawn(&scene, &target)
        .expect("unknown plugin type rejection belongs to isolated preflight");

    let error = capture_compiled_scene_spawn_preflight(&target, &plan, usize::MAX)
        .expect_err("a sparse preflight projection must preserve strict target catalog semantics");
    assert!(matches!(
        error,
        crate::scene::dynamic_scene::DynamicSceneError::WorldMutation(
            crate::scene::SceneError::UnregisteredDynamicComponentType { ref component_id }
        ) if component_id == UNKNOWN_TYPE_ID
    ));
}

#[test]
fn compiled_spawn_accepts_scene_declared_plugin_type_with_nonempty_target_catalog() {
    const TARGET_TYPE_ID: &str = "tests.TargetCatalogComponent";
    const SCENE_TYPE_ID: &str = "tests.SceneDeclaredComponent";
    let scene_descriptor = crate::core::framework::scene::ComponentTypeDescriptor::new(
        SCENE_TYPE_ID,
        "tests",
        "Scene Declared Component",
    );
    let mut source = World::empty();
    let source_entity = source.spawn_node(NodeKind::Empty);
    let mut scene = DynamicScene::from_world(&source).expect("source scene should capture");
    scene.component_types.push(scene_descriptor.clone());
    scene
        .entities
        .iter_mut()
        .find(|entity| entity.source_entity == source_entity)
        .expect("captured scene should retain the source entity")
        .components
        .push(DynamicComponent::new(SCENE_TYPE_ID, true, Vec::new()));

    let mut target = World::empty();
    target
        .register_component_type(crate::core::framework::scene::ComponentTypeDescriptor::new(
            TARGET_TYPE_ID,
            "tests",
            "Target Catalog Component",
        ))
        .expect("target-only component should make the catalog strict");

    scene
        .spawn_into(&mut target)
        .expect("a scene descriptor should declare its plugin write in strict catalog mode");
    assert_eq!(
        target.component_type_descriptor(SCENE_TYPE_ID),
        Some(&scene_descriptor)
    );
}

#[test]
fn compiled_spawn_commit_has_a_preflight_then_infallible_publication_boundary() {
    let manifest_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let commit_source = std::fs::read_to_string(
        manifest_root.join("src/scene/dynamic_scene/scene/spawn/commit.rs"),
    )
    .expect("read dynamic scene commit source");
    let world_transaction_source =
        std::fs::read_to_string(manifest_root.join("src/scene/world/transaction.rs"))
            .expect("read World transaction source");

    assert!(
        commit_source.contains("preflight_dynamic_scene_publication(")
            && commit_source.contains("publish_preflighted_dynamic_scene(publication);")
            && !commit_source.contains("install_component_type_descriptors("),
        "dynamic scene commit must complete all target-local row and descriptor preflight before publication"
    );
    let publication = world_transaction_source
        .split("pub(in crate::scene) fn publish_preflighted_dynamic_scene")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(in crate::scene) fn dynamic_scene_preflight_world")
                .next()
        })
        .expect("read preflighted dynamic scene publication body");
    assert!(
        !publication.contains("-> SceneResult")
            && !publication.contains("preflight_transferred_row(")
            && !publication.contains("registration_from_component_descriptor("),
        "preflighted publication must not retain a recoverable validation or conversion path"
    );
    let preflight = world_transaction_source
        .split("pub(in crate::scene) fn preflight_dynamic_scene_publication")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(in crate::scene) fn publish_preflighted_dynamic_scene")
                .next()
        })
        .expect("read dynamic scene publication preflight body");
    assert!(
        preflight.contains("if existing != &descriptor")
            && preflight.contains("SceneError::DuplicateComponentType")
            && !preflight.contains("debug_assert_eq!(existing, &descriptor)"),
        "an existing component descriptor must be release-validated before it is reused"
    );
}

struct TargetOnlyComponent;

impl Component for TargetOnlyComponent {}

#[test]
fn compiled_spawn_rekeys_preflight_component_rows_for_the_target_registry() {
    let mut source = World::empty();
    source.spawn_node(NodeKind::Empty);
    let scene = DynamicScene::from_world(&source).expect("source world should capture");

    let mut target = World::empty();
    target
        .spawn((TargetOnlyComponent,))
        .expect("target-only component should reserve its local registry slot");

    scene
        .spawn_into(&mut target)
        .expect("source component slots must be resolved in the target registry");
    assert_eq!(target.node_records().len(), 1);
}

#[test]
fn scene_spawn_keeps_target_unpublished_when_resource_write_preflight_fails() {
    let mut source = World::empty();
    register_rejecting_resource(&mut source);
    source.spawn_node(NodeKind::Empty);
    source.insert_resource(RejectingResource(7));
    let scene = DynamicScene::from_world(&source).expect("source scene should capture");

    let mut target = World::empty();
    register_rejecting_resource(&mut target);
    target.insert_resource(RejectingResource(3));
    let generation_before = target.world_generation();

    let error = scene
        .spawn_into(&mut target)
        .expect_err("a failing resource write must not publish scene records");

    assert!(matches!(
        error,
        crate::scene::dynamic_scene::DynamicSceneError::Reflect(
            ReflectError::UnsupportedConversion { .. }
        )
    ));
    assert!(target.node_records().is_empty());
    assert_eq!(
        target.get_resource::<RejectingResource>(),
        Some(&RejectingResource(3))
    );
    assert_eq!(target.world_generation(), generation_before);
}

#[test]
fn compiled_spawn_writes_resource_fields_through_dense_slots() {
    let mut source = World::empty();
    register_slot_resource(&mut source);
    source.insert_resource(SlotResource { value: 17 });
    let scene = DynamicScene::from_world(&source).expect("source scene should capture");

    let mut target = World::empty();
    register_slot_resource(&mut target);
    target.insert_resource(SlotResource { value: 3 });

    scene
        .spawn_into(&mut target)
        .expect("compiled scene should apply resource write");

    assert_eq!(
        target.get_resource::<SlotResource>(),
        Some(&SlotResource { value: 17 })
    );
}

#[test]
fn compiled_spawn_publishes_a_resource_preflight_result_without_replaying_the_adapter() {
    let mut source = World::empty();
    register_slot_resource(&mut source);
    source.spawn_node(NodeKind::Empty);
    source.insert_resource(SlotResource { value: 17 });
    let scene = DynamicScene::from_world(&source).expect("source scene should capture");

    let mut target = World::empty();
    register_slot_resource(&mut target);
    target.insert_resource(SlotResource { value: 3 });
    let generation_before = target.world_generation();
    let _write_budget = SlotResourceWriteBudgetReset::allow_exactly_one_write();

    scene
        .spawn_into(&mut target)
        .expect("the target must consume the preflight result without a second adapter write");

    assert_eq!(target.node_records().len(), 1);
    assert_eq!(
        target.get_resource::<SlotResource>(),
        Some(&SlotResource { value: 17 })
    );
    assert_ne!(target.world_generation(), generation_before);
}

#[cfg(feature = "profiling")]
#[test]
fn compiled_spawn_profiles_compact_commit_artifact_and_descriptor_delta() {
    let _capture_guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "dynamic-scene-compact-commit-artifact".to_string();
    config.max_counters = 64;
    start_capture(config);

    let mut source = World::empty();
    source.spawn_node(NodeKind::Empty);
    let scene = DynamicScene::from_world(&source).expect("source world should capture");
    let mut target = World::empty();

    scene
        .spawn_into(&mut target)
        .expect("compiled scene should publish its compact preflight artifact");

    let profile = snapshot();
    reset_capture();
    for name in [
        "dynamic_scene.transaction.commit_artifact.released_component_write_batches",
        "dynamic_scene.transaction.commit_artifact.released_resource_write_batches",
        "dynamic_scene.transaction.commit_artifact.materialized_component_rows",
        "dynamic_scene.transaction.component_registry.imported_descriptors",
        "dynamic_scene.transaction.component_registry.reused_descriptor_resolves",
    ] {
        assert!(
            profile.counters.iter().any(|counter| counter.name == name),
            "compiled scene profile should record `{name}`"
        );
    }
}

#[cfg(feature = "profiling")]
#[test]
fn preflight_schema_profile_counts_unique_affected_vm_catalog_path() {
    const TYPE_ID: &str = "tests.ProfiledVmComponent";
    const SHORT_TYPE_PATH: &str = "ProfiledVmComponent";
    let _capture_guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "dynamic-scene-affected-schema-projection".to_string();
    config.max_counters = 16;
    start_capture(config);

    let descriptor = crate::core::framework::scene::ComponentTypeDescriptor::new(
        TYPE_ID,
        "tests",
        "Profiled VM Component",
    );
    let registration = crate::scene::reflect::registration_from_component_descriptor(&descriptor)
        .expect("profiled VM registration must derive from its descriptor");
    let mut target = World::empty();
    target
        .sync_vm_types(std::slice::from_ref(&registration))
        .expect("profiled VM catalog should register");

    let preflight = target.dynamic_scene_preflight_world([SHORT_TYPE_PATH, TYPE_ID]);
    assert!(preflight.component_type_descriptor(TYPE_ID).is_some());
    assert!(preflight.type_registry().contains(TYPE_ID));
    assert!(preflight.is_vm_dynamic_type_path(TYPE_ID));

    let profile = snapshot();
    reset_capture();
    for name in [
        "dynamic_scene.transaction.preflight_schema.affected_type_paths",
        "dynamic_scene.transaction.preflight_schema.projected_component_descriptors",
        "dynamic_scene.transaction.preflight_schema.projected_runtime_registrations",
        "dynamic_scene.transaction.preflight_schema.projected_vm_catalog_type_paths",
        "dynamic_scene.transaction.preflight_schema.projected_vm_dynamic_type_paths",
    ] {
        let value = profile
            .counters
            .iter()
            .find(|counter| counter.stream == "runtime" && counter.name == name)
            .map(|counter| counter.value)
            .unwrap_or_else(|| panic!("missing preflight schema counter `{name}`"));
        assert_eq!(
            value, 1.0,
            "`{name}` must count the one canonical affected path, not the target catalog"
        );
    }
}
