use serde_json::json;
use std::path::{Path, PathBuf};

use crate::core::framework::scene::{ComponentPropertyDescriptor, ComponentTypeDescriptor};
use crate::core::math::{Transform, Vec3};
use crate::scene::components::{LocalTransform, Name, RenderLayerMask, RigidBodyComponent};
use crate::scene::ecs::{Component, Resource};
use crate::scene::{SceneError, World};

mod bundle_transactions;

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Mana(u32);

impl Component for Mana {}

#[derive(Debug, PartialEq, Eq)]
struct FrameCounter(u32);

impl Resource for FrameCounter {}

#[test]
fn world_spawn_insert_get_mut_and_remove_typed_components() {
    let mut world = World::empty();
    assert!(!world.contains_component::<Health>(u64::MAX));
    assert!(!world.is_component_added::<Health>(u64::MAX));
    assert!(!world.is_component_changed::<Health>(u64::MAX));

    let entity = world
        .spawn((
            Name("Typed Entity".to_string()),
            Health(7),
            LocalTransform {
                transform: Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            },
        ))
        .unwrap();

    assert!(world.contains_component::<Health>(entity));
    assert!(world.contains_component::<Name>(entity));
    assert!(world.is_component_added::<Health>(entity));
    assert!(world.is_component_changed::<Health>(entity));
    assert!(world.is_component_added::<Name>(entity));
    assert!(world.is_component_changed::<Name>(entity));
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

    world.clear_trackers();
    assert!(!world.is_component_added::<Health>(entity));
    assert!(!world.is_component_changed::<Health>(entity));
    assert!(!world.is_component_added::<Name>(entity));
    assert!(!world.is_component_changed::<Name>(entity));

    world.get_mut::<Health>(entity).unwrap().0 += 5;
    assert!(!world.is_component_added::<Health>(entity));
    assert!(world.is_component_changed::<Health>(entity));
    assert!(!world.is_component_changed::<Name>(entity));

    assert_eq!(world.insert(entity, Health(3)).unwrap(), Some(Health(12)));
    assert!(!world.is_component_added::<Health>(entity));
    assert!(world.is_component_changed::<Health>(entity));

    assert_eq!(world.remove::<Health>(entity).unwrap(), Some(Health(3)));
    assert!(!world.contains_component::<Health>(entity));
    assert!(!world.is_component_added::<Health>(entity));
    assert!(!world.is_component_changed::<Health>(entity));
    assert_eq!(world.get::<Health>(entity), None);

    world.clear_trackers();
    assert_eq!(world.insert(entity, Health(99)).unwrap(), None);
    assert!(world.contains_component::<Health>(entity));
    assert!(world.is_component_added::<Health>(entity));
    assert!(world.is_component_changed::<Health>(entity));

    world.clear_trackers();
    world
        .get_mut::<Name>(entity)
        .unwrap()
        .0
        .push_str(" Renamed");
    assert!(!world.is_component_added::<Name>(entity));
    assert!(world.is_component_changed::<Name>(entity));
}

#[test]
fn typed_local_transform_insertion_rejects_values_that_cannot_be_persisted() {
    let mut world = World::new();
    let entity = world.spawn_node(crate::scene::NodeKind::Mesh);
    let original = world
        .get::<LocalTransform>(entity)
        .expect("mesh nodes must have a local transform")
        .transform;
    let mut invalid = original;
    invalid.scale.z = 0.0;

    assert!(matches!(
        world.insert(entity, LocalTransform { transform: invalid }),
        Err(SceneError::ZeroScaleTransform { entity: error_entity, axis: "z" })
            if error_entity == entity
    ));
    assert_eq!(
        world.get::<LocalTransform>(entity).unwrap().transform,
        original
    );
}

#[test]
fn bundle_preflight_rejects_a_later_component_without_publishing_earlier_components() {
    let mut world = World::new();
    let entity = world.spawn_node(crate::scene::NodeKind::Mesh);
    let original_transform = world
        .get::<LocalTransform>(entity)
        .expect("mesh nodes must have a local transform")
        .transform;
    let generation_before = world.world_generation();
    let mut invalid_transform = original_transform;
    invalid_transform.scale.z = 0.0;

    assert!(matches!(
        world.insert_bundle(
            entity,
            (
                Health(42),
                LocalTransform {
                    transform: invalid_transform,
                },
            ),
        ),
        Err(SceneError::ZeroScaleTransform {
            entity: error_entity,
            axis: "z",
        }) if error_entity == entity
    ));

    assert!(!world.contains_component::<Health>(entity));
    assert_eq!(world.registered_component_id::<Health>(), None);
    assert_eq!(
        world
            .get::<LocalTransform>(entity)
            .map(|component| component.transform),
        Some(original_transform)
    );
    assert_eq!(world.world_generation(), generation_before);
}

#[test]
fn bundle_spawn_preflight_does_not_publish_a_default_entity_on_failure() {
    let mut world = World::empty();
    let generation_before = world.world_generation();
    let mut invalid_transform = Transform::default();
    invalid_transform.scale.z = 0.0;

    assert!(matches!(
        world.spawn((
            Health(42),
            LocalTransform {
                transform: invalid_transform,
            },
        )),
        Err(SceneError::ZeroScaleTransform {
            entity: 1,
            axis: "z",
        })
    ));

    assert!(world.node_record(1).is_none());
    assert!(!world.contains_component::<Health>(1));
    assert_eq!(world.registered_component_id::<Health>(), None);
    assert_eq!(world.world_generation(), generation_before);
}

#[test]
fn bundle_preflight_rejects_duplicate_component_types_without_publishing() {
    let mut world = World::empty();
    let generation_before = world.world_generation();

    assert!(matches!(
        world.spawn((Health(7), Health(9))),
        Err(SceneError::Message(message)) if message == "bundle cannot contain duplicate component types"
    ));

    assert!(world.node_record(1).is_none());
    assert_eq!(world.registered_component_id::<Health>(), None);
    assert_eq!(world.world_generation(), generation_before);
}

#[test]
fn bundle_spawn_preflights_default_node_and_custom_component_ids_together() {
    let mut world = World::empty();

    let entity = world
        .spawn((Health(7),))
        .expect("bundle spawn must assign custom ids after default node components");

    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert!(world.node_record(entity).is_some());
}

#[test]
fn spawn_node_lifecycle_observers_see_the_final_fixed_component_signature() {
    use std::sync::{Arc, Mutex};

    let mut world = World::empty();
    let local_transform_id = world.component_id::<LocalTransform>();
    let saw_final_signature = Arc::new(Mutex::new(false));
    let saw_final_signature_from_observer = Arc::clone(&saw_final_signature);
    world.observe_component_lifecycle::<Name>(
        crate::scene::ecs::LifecycleEventKind::Add,
        move |world, event| {
            *saw_final_signature_from_observer
                .lock()
                .expect("observer state lock") = world
                .node_record(event.entity())
                .is_some_and(|_| world.contains_component_id(event.entity(), local_transform_id));
        },
    );

    world.spawn_node(crate::scene::NodeKind::Empty);

    assert!(*saw_final_signature.lock().expect("observer state lock"));
}

#[test]
fn bundle_lifecycle_observers_see_the_final_component_signature() {
    use std::sync::{Arc, Mutex};

    let mut world = World::empty();
    let entity = world.spawn_node(crate::scene::NodeKind::Empty);
    let mana_id = world.component_id::<Mana>();
    let saw_final_signature = Arc::new(Mutex::new(false));
    let saw_final_signature_from_observer = Arc::clone(&saw_final_signature);
    world.observe_component_lifecycle::<Health>(
        crate::scene::ecs::LifecycleEventKind::Add,
        move |world, _event| {
            *saw_final_signature_from_observer
                .lock()
                .expect("observer state lock") = world.contains_component_id(entity, mana_id);
        },
    );

    world
        .insert_bundle(entity, (Health(7), Mana(9)))
        .expect("validated bundle must commit");

    assert!(*saw_final_signature.lock().expect("observer state lock"));
}

#[test]
fn bundle_spawn_lifecycle_observers_see_the_final_component_signature() {
    use std::sync::{Arc, Mutex};

    let mut world = World::empty();
    let mana_id = world.component_id::<Mana>();
    let saw_final_signature = Arc::new(Mutex::new(false));
    let saw_final_signature_from_observer = Arc::clone(&saw_final_signature);
    world.observe_component_lifecycle::<Health>(
        crate::scene::ecs::LifecycleEventKind::Add,
        move |world, event| {
            *saw_final_signature_from_observer
                .lock()
                .expect("observer state lock") = world
                .node_record(event.entity())
                .is_some_and(|_| world.contains_component_id(event.entity(), mana_id));
        },
    );

    let entity = world
        .spawn((Health(7), Mana(9)))
        .expect("validated bundle spawn must commit");

    assert_eq!(entity, 1);
    assert!(*saw_final_signature.lock().expect("observer state lock"));
}

#[test]
fn world_resources_are_registered_and_replaced_by_type() {
    let mut world = World::empty();

    assert!(!world.contains_resource::<FrameCounter>());
    assert!(!world.is_resource_added::<FrameCounter>());
    assert!(!world.is_resource_changed::<FrameCounter>());

    let resource_id = world.resource_id::<FrameCounter>();
    assert_eq!(resource_id.index(), 0);
    assert_eq!(world.insert_resource(FrameCounter(1)), None);
    assert_eq!(world.resource::<FrameCounter>(), &FrameCounter(1));
    assert!(world.contains_resource::<FrameCounter>());
    assert!(world.is_resource_added::<FrameCounter>());
    assert!(world.is_resource_changed::<FrameCounter>());

    world.clear_trackers();
    assert!(!world.is_resource_added::<FrameCounter>());
    assert!(!world.is_resource_changed::<FrameCounter>());

    world.resource_mut::<FrameCounter>().0 += 1;
    assert!(!world.is_resource_added::<FrameCounter>());
    assert!(world.is_resource_changed::<FrameCounter>());

    assert_eq!(
        world.insert_resource(FrameCounter(9)),
        Some(FrameCounter(2))
    );
    assert_eq!(world.resource::<FrameCounter>(), &FrameCounter(9));
    assert!(!world.is_resource_added::<FrameCounter>());
    assert!(world.is_resource_changed::<FrameCounter>());

    assert_eq!(
        world.remove_resource::<FrameCounter>(),
        Some(FrameCounter(9))
    );
    assert!(!world.contains_resource::<FrameCounter>());
    assert!(!world.is_resource_added::<FrameCounter>());
    assert!(!world.is_resource_changed::<FrameCounter>());

    world.clear_trackers();
    assert_eq!(world.insert_resource(FrameCounter(4)), None);
    assert!(world.is_resource_added::<FrameCounter>());
    assert!(world.is_resource_changed::<FrameCounter>());
    assert_eq!(
        world.registered_resource_id::<FrameCounter>(),
        Some(resource_id)
    );
}

#[test]
fn world_typed_mutation_errors_report_missing_entities_as_scene_errors() {
    let mut world = World::empty();
    let missing = u64::MAX;

    assert_eq!(
        world.insert(missing, Health(1)),
        Err(SceneError::MissingEntity {
            operation: "insert component on",
            entity: missing
        })
    );
    assert_eq!(
        world.insert_bundle(missing, (Health(2),)),
        Err(SceneError::MissingEntity {
            operation: "insert component on",
            entity: missing
        })
    );
    assert_eq!(
        world.remove::<Health>(missing),
        Err(SceneError::MissingEntity {
            operation: "remove component from",
            entity: missing
        })
    );
}

#[test]
fn dynamic_component_mutation_errors_report_scene_error_variants() {
    let mut world = World::empty();
    let missing = u64::MAX;

    assert_eq!(
        world.set_dynamic_component(missing, "weather.cloud", json!({})),
        Err(SceneError::MissingEntity {
            operation: "attach dynamic component to",
            entity: missing
        })
    );
    assert_eq!(
        world.register_component_type(ComponentTypeDescriptor::new("cloud", "weather", "Cloud")),
        Err(SceneError::ComponentTypePluginPrefixMismatch {
            type_id: "cloud".to_string(),
            plugin_id: "weather".to_string()
        })
    );

    let entity = world.spawn((Name("Dynamic Entity".to_string()),)).unwrap();
    world
        .register_component_type(ComponentTypeDescriptor::new(
            "weather.cloud",
            "weather",
            "Cloud",
        ))
        .unwrap();

    assert_eq!(
        world.set_dynamic_component(entity, "weather.rain", json!({})),
        Err(SceneError::UnregisteredDynamicComponentType {
            component_id: "weather.rain".to_string()
        })
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
fn world_archetype_rebuild_walks_stable_entity_ids_without_cloning_list() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("identity.rs"),
    );
    let rebuild = source
        .split("pub(super) fn rebuild_archetype_index")
        .nth(1)
        .and_then(|text| text.split("fn assign_entity_archetype").next())
        .expect("read archetype index rebuild body");

    assert!(rebuild.contains("for entity_index in 0..self.entities.len()"));
    assert!(rebuild.contains("let entity = self.entities[entity_index];"));
    assert!(rebuild.contains("self.assign_entity_archetype_from_component_storage(entity, None);"));
    assert!(!rebuild.contains("self.entities.clone()"));
}

#[test]
fn archetype_signatures_partition_component_ids_without_combined_vector() {
    let identity_source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("identity.rs"),
    );
    let storage_source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("ecs")
            .join("storage")
            .join("component_storage")
            .join("store.rs"),
    );
    let signature = identity_source
        .split("fn archetype_signature_for_internal")
        .nth(1)
        .and_then(|text| text.split("fn entity_registry_error_to_string").next())
        .expect("read archetype signature body");
    let partition = storage_source
        .split("pub(crate) fn component_ids_for_entity_by_storage")
        .nth(1)
        .and_then(|text| text.split("fn component_storage_count").next())
        .expect("read component id partition helper body");

    assert!(
        signature.contains("self.component_storage.component_ids_for_entity_by_storage(")
            && signature.contains("&mut table_components")
            && signature.contains("&mut sparse_set_components")
            && signature
                .contains("ArchetypeSignature::new(table_components, sparse_set_components)")
            && !signature.contains("component_ids_for_entity(internal)")
            && !signature.contains("self.component_storage.storage_type(component_id)")
            && !signature.contains("match self.component_storage.storage_type"),
        "archetype signatures must ask component storage for table/sparse buckets directly instead of building one combined component-id vector and querying storage types again"
    );
    assert!(
        partition.contains("table_components.clear();")
            && partition.contains("table_components.reserve(self.table_components.len());")
            && partition.contains("for (component_id, storage) in &self.table_components")
            && partition.contains("table_components.push(*component_id);")
            && partition.contains("sparse_set_components.clear();")
            && partition.contains("sparse_set_components.reserve(self.sparse_components.len());")
            && partition.contains("for (component_id, storage) in &self.sparse_components")
            && partition.contains("sparse_set_components.push(*component_id);")
            && !partition.contains("self.storage_types")
            && !partition.contains("storage_type("),
        "component storage must fill caller-owned table/sparse component-id buckets directly from storage maps without storage-type map lookups"
    );
}

#[test]
fn typed_component_presence_rebuild_reuses_dynamic_component_id_scratch() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api.rs"),
    );
    let rebuild = source
        .split("pub(super) fn rebuild_typed_component_presence")
        .nth(1)
        .and_then(|text| {
            text.split("fn dynamic_component_type_ids_for_presence_rebuild")
                .next()
        })
        .expect("read typed component presence rebuild body");
    let dynamic_ids = source
        .split("fn dynamic_component_type_ids_for_presence_rebuild")
        .nth(1)
        .and_then(|text| text.split("fn mark_component_mutation").next())
        .expect("read dynamic component id scratch helper body");

    assert!(
        rebuild.contains("let mut dynamic_component_type_ids = Vec::new();")
            && rebuild.contains("for entity_index in 0..self.entities.len()")
            && rebuild.contains("let entity = self.entities[entity_index];")
            && rebuild.contains("self.dynamic_component_type_ids_for_presence_rebuild(")
            && rebuild.contains("for component_type_id in &dynamic_component_type_ids")
            && rebuild
                .contains("self.insert_dynamic_component_presence(entity, component_type_id)")
            && !rebuild.contains("self.entities.clone()")
            && !rebuild.contains("self.dynamic_components.get(&entity).cloned()"),
        "typed component presence rebuild must walk stable entity ids by index and reuse a dynamic component id scratch list"
    );
    assert!(
        dynamic_ids.contains("output.clear();")
            && dynamic_ids.contains("let Some(components) = self.dynamic_components.get(&entity)")
            && dynamic_ids.contains("output.reserve(components.len());")
            && dynamic_ids.contains("for component_type_id in components.keys()")
            && dynamic_ids.contains("output.push(component_type_id.clone());")
            && !dynamic_ids.contains("value.clone()")
            && !dynamic_ids.contains(".cloned()"),
        "dynamic component presence rebuild must clone only component ids into caller-owned scratch storage, not whole component maps or JSON values"
    );
}

#[test]
fn typed_world_presence_and_tracker_helpers_use_direct_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api.rs"),
    );
    let contains_component_id = source
        .split("pub fn contains_component_id")
        .nth(1)
        .and_then(|text| text.split("pub fn contains_component<").next())
        .expect("read contains_component_id body");
    let contains_component = source
        .split("pub fn contains_component<")
        .nth(1)
        .and_then(|text| text.split("pub fn is_component_added").next())
        .expect("read contains_component body");
    let is_component_added = source
        .split("pub fn is_component_added")
        .nth(1)
        .and_then(|text| text.split("pub fn is_component_changed").next())
        .expect("read is_component_added body");
    let is_component_changed = source
        .split("pub fn is_component_changed")
        .nth(1)
        .and_then(|text| text.split("pub fn insert<T>").next())
        .expect("read is_component_changed body");
    let remove = source
        .split("pub fn remove<T>")
        .nth(1)
        .and_then(|text| text.split("pub fn resource_id").next())
        .expect("read remove body");
    let is_resource_added = source
        .split("pub fn is_resource_added")
        .nth(1)
        .and_then(|text| text.split("pub fn is_resource_changed").next())
        .expect("read is_resource_added body");
    let is_resource_changed = source
        .split("pub fn is_resource_changed")
        .nth(1)
        .and_then(|text| text.split("pub fn insert_resource").next())
        .expect("read is_resource_changed body");
    let get_resource_mut = source
        .split("pub fn get_resource_mut")
        .nth(1)
        .and_then(|text| text.split("pub fn remove_resource").next())
        .expect("read get_resource_mut body");

    assert!(
        contains_component_id.contains("let Some(internal) = self.internal_entity(entity) else")
            && contains_component_id.contains("return false;")
            && contains_component_id
                .contains("self.component_storage.contains(component_id, internal)")
            && !contains_component_id.contains(".is_some_and(")
            && contains_component
                .contains("let Some(component_id) = self.registered_component_id::<T>() else")
            && contains_component.contains("self.contains_component_id(entity, component_id)")
            && !contains_component.contains(".is_some_and(")
            && is_component_added
                .contains("let Some(ticks) = self.component_change_ticks::<T>(entity) else")
            && is_component_added
                .contains("ticks.is_added(crate::scene::ecs::ChangeTickWindow::new")
            && !is_component_added.contains(".is_some_and(")
            && is_component_changed
                .contains("let Some(ticks) = self.component_change_ticks::<T>(entity) else")
            && is_component_changed
                .contains("ticks.is_changed(crate::scene::ecs::ChangeTickWindow::new")
            && !is_component_changed.contains(".is_some_and(")
            && remove.contains("if let Some(component_id) = component_id")
            && remove.contains("if self.contains_component_id(entity, component_id)")
            && !remove.contains("component_id\n                .is_some_and(")
            && !remove.contains("component_id.expect(\"checked component id must be present\")")
            && is_resource_added
                .contains("let Some(ticks) = self.resource_change_ticks::<T>() else")
            && is_resource_added
                .contains("ticks.is_added(crate::scene::ecs::ChangeTickWindow::new")
            && !is_resource_added.contains(".is_some_and(")
            && is_resource_changed
                .contains("let Some(ticks) = self.resource_change_ticks::<T>() else")
            && is_resource_changed
                .contains("ticks.is_changed(crate::scene::ecs::ChangeTickWindow::new")
            && !is_resource_changed.contains(".is_some_and(")
            && get_resource_mut.contains(
                "let Some((resource, ticks, tick)) = self.resource_mut_with_ticks::<T>() else"
            )
            && get_resource_mut.contains("ticks.set_changed(tick);")
            && get_resource_mut.contains("Some(resource)")
            && !get_resource_mut.contains(".map(|(resource, ticks, tick)| resource)"),
        "typed world presence and tracker helpers must use direct branches instead of Option adapter predicates"
    );
}

#[test]
fn typed_world_component_insert_remove_use_direct_result_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api.rs"),
    );
    let insert = source
        .split("pub fn insert<T>")
        .nth(1)
        .and_then(|text| text.split("pub fn get<T>").next())
        .expect("read typed component insert body");
    let remove = source
        .split("pub fn remove<T>")
        .nth(1)
        .and_then(|text| text.split("pub fn resource_id").next())
        .expect("read typed component remove body");

    assert!(
        insert.contains("let old = match self")
            && insert.contains("insert_at_tick(")
            && insert.contains("T::STORAGE_TYPE")
            && insert.contains("internal")
            && insert.contains("component")
            && insert.contains("tick")
            && insert.contains("Ok(old) => old")
            && insert.contains("Err(error) => return Err(error.into())")
            && !insert.contains(".map_err(|error| error.to_string())")
            && remove.contains("match self.component_storage.remove::<T>(component_id, internal)")
            && remove.contains("Ok(_) => {}")
            && remove.contains("Ok(Some(ComponentRemoveResult { value, .. })) => Some(value)")
            && remove.contains("Ok(None) => None")
            && remove.contains("Err(error) => return Err(error.into())")
            && !remove.contains(".map_err(|error| error.to_string())")
            && !remove.contains(".map(|ComponentRemoveResult { value, .. }| value)"),
        "typed component insert/remove must use direct Result branches instead of map_err/map adapters"
    );
}

#[test]
fn dynamic_component_presence_updates_use_direct_result_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api.rs"),
    );
    let insert_presence = source
        .split("pub(super) fn insert_dynamic_component_presence")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn remove_dynamic_component_presence")
                .next()
        })
        .expect("read dynamic component presence insert body");
    let remove_presence = source
        .split("pub(super) fn remove_dynamic_component_presence")
        .nth(1)
        .and_then(|text| {
            text.split("pub(super) fn rebuild_typed_component_presence")
                .next()
        })
        .expect("read dynamic component presence remove body");

    assert!(
        insert_presence.contains("let old = self.component_storage.insert_at_tick(")
            && insert_presence.contains("DynamicComponentPresence")
            && insert_presence.contains(")?;")
            && !insert_presence.contains(".map_err(|error| error.to_string())")
            && remove_presence.contains("let removed = self")
            && remove_presence
                .contains(".remove::<DynamicComponentPresence>(component_id, internal)")
            && remove_presence.contains(")?;")
            && !remove_presence.contains(".map_err(|error| error.to_string())"),
        "dynamic component presence updates must propagate typed SceneResult errors instead of map_err adapters"
    );
}

#[test]
fn typed_world_required_resource_accessors_use_direct_missing_branches() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("typed_api.rs"),
    );
    let resource = source
        .split("pub fn resource<T>")
        .nth(1)
        .and_then(|text| text.split("pub fn get_resource<T>").next())
        .expect("read required resource accessor body")
        .replace("\r\n", "\n");
    let resource_mut = source
        .split("pub fn resource_mut<T>")
        .nth(1)
        .and_then(|text| text.split("pub fn get_resource_mut<T>").next())
        .expect("read required mutable resource accessor body")
        .replace("\r\n", "\n");

    assert!(
        resource.contains("let Some(resource) = self.get_resource::<T>() else")
            && resource.contains("requested missing scene resource {}")
            && resource.contains("std::any::type_name::<T>()")
            && resource.contains("};\n\n        resource")
            && !resource.contains(".unwrap_or_else(")
            && resource_mut.contains("let Some(resource) = self.get_resource_mut::<T>() else")
            && resource_mut.contains("requested missing scene resource {}")
            && resource_mut.contains("std::any::type_name::<T>()")
            && resource_mut.contains("};\n\n        resource")
            && !resource_mut.contains(".unwrap_or_else("),
        "typed required resource accessors must use direct missing-resource branches instead of unwrap_or_else closures"
    );
}

#[test]
fn world_set_joint_self_connection_uses_direct_option_branch() {
    let source = read_source(
        &manifest_dir()
            .join("src")
            .join("scene")
            .join("world")
            .join("component_access.rs"),
    );
    let set_joint = source
        .split("pub fn set_joint")
        .nth(1)
        .and_then(|text| text.split("pub fn set_point_light").next())
        .expect("read set_joint body");

    assert!(
        set_joint.contains("let joint_connects_to_self = match &joint")
            && set_joint.contains("Some(joint) => joint.connected_entity == Some(entity)")
            && set_joint.contains("None => false")
            && set_joint.contains("if joint_connects_to_self")
            && !set_joint.contains(".and_then(|joint| joint.connected_entity)"),
        "World::set_joint must use a direct Option branch for self-connection validation"
    );
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

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_source(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}
