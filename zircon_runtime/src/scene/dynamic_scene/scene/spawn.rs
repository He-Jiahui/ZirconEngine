use std::collections::{BTreeSet, HashMap};

use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

use crate::scene::components::NodeRecord;
use crate::scene::{EntityId, World};

use super::DynamicScene;
use crate::scene::dynamic_scene::value::{reflected_fields_to_json_object, remap_reflected_value};
use crate::scene::dynamic_scene::{
    DynamicComponent, DynamicResource, DynamicSceneError, EntityRemap,
    ScenePatchPreviewComponentType, ScenePatchPreviewEntityRemap, ScenePatchPreviewReport,
    ScenePatchPreviewResource,
};

/// Owns the target-bound resolution work so apply never reparses a scene payload.
#[derive(Clone)]
pub(crate) struct CompiledSceneSpawn {
    expected_target_generation: u64,
    expected_schema_catalog_generation: u64,
    remap: EntityRemap,
    records: Vec<NodeRecord>,
    component_writes: Vec<CompiledComponentWrite>,
    resource_writes: Vec<CompiledResourceWrite>,
    preview: ScenePatchPreviewReport,
}

#[derive(Clone)]
struct CompiledComponentWrite {
    entity: EntityId,
    kind: CompiledComponentWriteKind,
}

#[derive(Clone)]
enum CompiledComponentWriteKind {
    Plugin {
        type_path: String,
        value: serde_json::Value,
    },
    Reflected {
        adapter: crate::scene::reflect::ReflectComponent,
        writes: Vec<(u32, ReflectedValue)>,
    },
}

#[derive(Clone)]
struct CompiledResourceWrite {
    type_path: String,
    adapter: crate::scene::reflect::ReflectResource,
    writes: Vec<(String, ReflectedValue)>,
}

impl CompiledSceneSpawn {
    fn into_preview(self) -> ScenePatchPreviewReport {
        self.preview
    }
}

pub(super) fn spawn_scene_into(
    scene: &DynamicScene,
    world: &mut World,
) -> Result<EntityRemap, DynamicSceneError> {
    let plan = compile_scene_spawn(scene, world)?;
    apply_compiled_scene_spawn(scene, world, plan)
}

pub(crate) fn compile_scene_spawn(
    scene: &DynamicScene,
    world: &World,
) -> Result<CompiledSceneSpawn, DynamicSceneError> {
    scene.ensure_supported()?;
    ensure_component_type_descriptors_are_compatible(scene, world)?;
    let remap = build_entity_remap(scene, world)?;
    let records = compile_entity_records(scene, world, &remap)?;
    world.validate_owned_node_records(&records)?;
    let component_writes = compile_component_writes(scene, world, &remap)?;
    let (resource_writes, resources) = compile_resource_writes(scene, world, &remap)?;
    let preview = build_preview_report(scene, world, &remap, resources);
    Ok(CompiledSceneSpawn {
        expected_target_generation: world.world_generation(),
        expected_schema_catalog_generation: world.type_registry().schema_catalog_generation(),
        remap,
        records,
        component_writes,
        resource_writes,
        preview,
    })
}

pub(crate) fn apply_compiled_scene_spawn(
    scene: &DynamicScene,
    world: &mut World,
    plan: CompiledSceneSpawn,
) -> Result<EntityRemap, DynamicSceneError> {
    ensure_compiled_spawn_target_is_current(world, &plan)?;
    let (mut preflight, _) =
        capture_compiled_scene_spawn_preflight(scene, world, &plan, usize::MAX)?;
    validate_compiled_scene_spawn_preflight(scene, &mut preflight, &plan)?;
    apply_preflighted_compiled_scene_spawn(scene, world, plan)
}

pub(crate) fn apply_preflighted_compiled_scene_spawn(
    scene: &DynamicScene,
    world: &mut World,
    plan: CompiledSceneSpawn,
) -> Result<EntityRemap, DynamicSceneError> {
    ensure_compiled_spawn_target_is_current(world, &plan)?;
    apply_compiled_scene_spawn_unchecked(scene, world, plan)
}

fn ensure_compiled_spawn_target_is_current(
    world: &World,
    plan: &CompiledSceneSpawn,
) -> Result<(), DynamicSceneError> {
    let actual_schema_generation = world.type_registry().schema_catalog_generation();
    if actual_schema_generation != plan.expected_schema_catalog_generation {
        return Err(DynamicSceneError::TargetSchemaChanged {
            expected_generation: plan.expected_schema_catalog_generation,
            actual_generation: actual_schema_generation,
        });
    }
    let actual_generation = world.world_generation();
    if actual_generation != plan.expected_target_generation {
        return Err(DynamicSceneError::TargetWorldChanged {
            expected_generation: plan.expected_target_generation,
            actual_generation,
        });
    }
    Ok(())
}

/// Captures the bounded, isolated World needed to validate a compiled scene
/// mutation. No target entity or component storage is cloned or modified.
pub(crate) fn capture_compiled_scene_spawn_preflight(
    scene: &DynamicScene,
    target: &World,
    plan: &CompiledSceneSpawn,
    limit_bytes: usize,
) -> Result<(World, usize), DynamicSceneError> {
    let mut preflight = target.dynamic_scene_preflight_world();
    insert_preflight_parent_context(target, &mut preflight, &plan.records)?;
    let base_estimated_bytes = preflight.estimate_dynamic_scene_preflight_bytes(limit_bytes)?;
    let estimated_bytes = scene.stage_existing_resources_bounded(
        target,
        &mut preflight,
        base_estimated_bytes,
        limit_bytes,
    )?;
    Ok((preflight, estimated_bytes))
}

/// Runs every fallible reflected write against the captured preflight World.
/// The target mutation can consume the same compiled plan only after this
/// returns successfully.
pub(crate) fn validate_compiled_scene_spawn_preflight(
    scene: &DynamicScene,
    preflight: &mut World,
    plan: &CompiledSceneSpawn,
) -> Result<(), DynamicSceneError> {
    apply_compiled_scene_spawn_unchecked(scene, preflight, plan.clone()).map(|_| ())
}

fn insert_preflight_parent_context(
    target: &World,
    preflight: &mut World,
    records: &[NodeRecord],
) -> Result<(), DynamicSceneError> {
    let planned_entities = records
        .iter()
        .map(|record| record.id)
        .collect::<BTreeSet<_>>();
    let mut context = Vec::new();
    let mut visited = BTreeSet::new();
    for record in records {
        let mut parent = record.parent;
        while let Some(entity) = parent {
            if planned_entities.contains(&entity) || !visited.insert(entity) {
                break;
            }
            let Some(record) = target.node_record(entity) else {
                break;
            };
            parent = record.parent;
            context.push(record);
        }
    }
    preflight
        .insert_owned_node_records(context)
        .map_err(Into::into)
}

fn apply_compiled_scene_spawn_unchecked(
    scene: &DynamicScene,
    world: &mut World,
    plan: CompiledSceneSpawn,
) -> Result<EntityRemap, DynamicSceneError> {
    let CompiledSceneSpawn {
        remap,
        records,
        component_writes,
        resource_writes,
        expected_target_generation: _,
        expected_schema_catalog_generation: _,
        preview: _,
    } = plan;
    install_component_type_descriptors(scene, world)?;
    insert_entity_records(world, records)?;
    apply_component_writes(world, component_writes)?;
    apply_resource_writes(world, resource_writes)?;
    Ok(remap)
}

pub(super) fn preview_scene_spawn_into(
    scene: &DynamicScene,
    world: &World,
) -> Result<ScenePatchPreviewReport, DynamicSceneError> {
    Ok(compile_scene_spawn(scene, world)?.into_preview())
}

fn build_preview_report(
    scene: &DynamicScene,
    world: &World,
    remap: &EntityRemap,
    resources: Vec<ScenePatchPreviewResource>,
) -> ScenePatchPreviewReport {
    let component_type_count = scene.component_types.len();
    let existing_component_type_count = scene
        .component_types
        .iter()
        .filter(|descriptor| {
            world
                .component_type_descriptor(&descriptor.type_id)
                .is_some()
        })
        .count();
    let new_component_type_count =
        component_type_count.saturating_sub(existing_component_type_count);
    let component_types = scene
        .component_types
        .iter()
        .map(|descriptor| ScenePatchPreviewComponentType {
            type_id: descriptor.type_id.clone(),
            plugin_id: descriptor.plugin_id.clone(),
            display_name: descriptor.display_name.clone(),
            already_registered: world
                .component_type_descriptor(&descriptor.type_id)
                .is_some(),
        })
        .collect();
    let remapped_entity_count = remap
        .iter()
        .filter(|(source, target)| source != target)
        .count();
    let entity_remaps = remap
        .iter()
        .map(
            |(source_entity, target_entity)| ScenePatchPreviewEntityRemap {
                source_entity,
                target_entity,
            },
        )
        .collect();
    let entity_count = scene.entities.len();
    let component_instance_count: usize = scene
        .entities
        .iter()
        .map(|entity| entity.components.len())
        .sum();
    ScenePatchPreviewReport {
        component_type_count,
        existing_component_type_count,
        new_component_type_count,
        component_instance_count,
        entity_count,
        resource_count: scene.resources.len(),
        target_entity_count: world.node_records().len(),
        preserved_entity_count: entity_count.saturating_sub(remapped_entity_count),
        remapped_entity_count,
        component_types,
        resources,
        entity_remaps,
    }
}

pub(super) fn stage_existing_resources_bounded(
    scene: &DynamicScene,
    source: &World,
    target: &mut World,
    base_estimated_bytes: usize,
    limit_bytes: usize,
) -> Result<usize, DynamicSceneError> {
    let mut estimated_bytes = base_estimated_bytes;
    for resource in &scene.resources {
        let runtime = source
            .type_registry()
            .runtime_registration(&resource.type_path)?;
        let adapter = runtime
            .resource
            .ok_or_else(|| ReflectError::NoResourceAdapter {
                type_path: resource.type_path.clone(),
            })?;
        if adapter.contains(source) {
            let resource_bytes = adapter.estimate_stage_clone_bytes(source)?.ok_or_else(|| {
                DynamicSceneError::MissingResourceStagingSizeEstimate {
                    type_path: resource.type_path.clone(),
                }
            })?;
            estimated_bytes = estimated_bytes.saturating_add(resource_bytes);
            if estimated_bytes > limit_bytes {
                return Err(DynamicSceneError::TargetSnapshotTooLarge {
                    estimated_bytes,
                    limit_bytes,
                });
            }
            if !adapter.stage_clone(source, target)? {
                return Err(DynamicSceneError::MissingResourceStagingClone {
                    type_path: resource.type_path.clone(),
                });
            }
        }
    }
    Ok(estimated_bytes)
}

fn install_component_type_descriptors(
    scene: &DynamicScene,
    world: &mut World,
) -> Result<(), DynamicSceneError> {
    for descriptor in &scene.component_types {
        if let Some(existing) = world.component_type_descriptor(&descriptor.type_id) {
            if existing != descriptor {
                return Err(DynamicSceneError::ComponentTypeDescriptorConflict {
                    type_id: descriptor.type_id.clone(),
                });
            }
            continue;
        }
        world.register_component_type(descriptor.clone())?;
    }
    Ok(())
}

fn ensure_component_type_descriptors_are_compatible(
    scene: &DynamicScene,
    world: &World,
) -> Result<(), DynamicSceneError> {
    for descriptor in &scene.component_types {
        if let Some(existing) = world.component_type_descriptor(&descriptor.type_id) {
            if existing != descriptor {
                return Err(DynamicSceneError::ComponentTypeDescriptorConflict {
                    type_id: descriptor.type_id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn compile_component_writes(
    scene: &DynamicScene,
    world: &World,
    remap: &EntityRemap,
) -> Result<Vec<CompiledComponentWrite>, DynamicSceneError> {
    let component_count = scene
        .entities
        .iter()
        .map(|entity| entity.components.len())
        .sum();
    let mut writes = Vec::with_capacity(component_count);
    for entity in &scene.entities {
        let target = remapped_entity(remap, entity.source_entity)?;
        for component in &entity.components {
            writes.push(compile_component_write(world, target, component, remap)?);
        }
    }
    Ok(writes)
}

fn compile_component_write(
    world: &World,
    entity: EntityId,
    component: &DynamicComponent,
    remap: &EntityRemap,
) -> Result<CompiledComponentWrite, DynamicSceneError> {
    if component.plugin_owned {
        return Ok(CompiledComponentWrite {
            entity,
            kind: CompiledComponentWriteKind::Plugin {
                type_path: component.type_path.clone(),
                value: reflected_fields_to_json_object(&component.fields, remap)?,
            },
        });
    }

    let runtime = world
        .type_registry()
        .runtime_registration(&component.type_path)?;
    if !runtime.registration.is_component {
        return Err(ReflectError::AddressKindMismatch {
            expected: format!("component `{}`", component.type_path),
            actual: format!("non-component `{}`", component.type_path),
        }
        .into());
    }
    let adapter = runtime
        .component
        .clone()
        .ok_or_else(|| ReflectError::NoComponentAdapter {
            type_path: component.type_path.clone(),
        })?;

    let fields = &runtime.registration.type_info.fields;
    let mut field_slots = HashMap::with_capacity(fields.len());
    for (field_slot, field) in fields.iter().enumerate() {
        let field_slot =
            u32::try_from(field_slot).map_err(|_| ReflectError::InvalidRegistration {
                type_path: component.type_path.clone(),
                reason: "component reflection has more than u32::MAX fields".to_string(),
            })?;
        field_slots.insert(
            field.name.as_str(),
            (field_slot, field.serializable && field.editable),
        );
    }

    let mut writes = Vec::with_capacity(component.fields.len());
    for field in &component.fields {
        let Some((field_slot, writable)) = field_slots.get(field.field_name.as_str()).copied()
        else {
            return Err(ReflectError::UnknownField {
                type_path: component.type_path.clone(),
                field_name: field.field_name.clone(),
            }
            .into());
        };
        if writable {
            writes.push((field_slot, remap_reflected_value(&field.value, remap)?));
        }
    }
    Ok(CompiledComponentWrite {
        entity,
        kind: CompiledComponentWriteKind::Reflected { adapter, writes },
    })
}

fn compile_resource_writes(
    scene: &DynamicScene,
    world: &World,
    remap: &EntityRemap,
) -> Result<(Vec<CompiledResourceWrite>, Vec<ScenePatchPreviewResource>), DynamicSceneError> {
    let mut writes = Vec::with_capacity(scene.resources.len());
    let mut previews = Vec::with_capacity(scene.resources.len());
    for resource in &scene.resources {
        let (write, preview) = compile_resource_write(world, resource, remap)?;
        writes.push(write);
        previews.push(preview);
    }
    Ok((writes, previews))
}

fn compile_resource_write(
    world: &World,
    resource: &DynamicResource,
    remap: &EntityRemap,
) -> Result<(CompiledResourceWrite, ScenePatchPreviewResource), DynamicSceneError> {
    let runtime = world
        .type_registry()
        .runtime_registration(&resource.type_path)?;
    if !runtime.registration.is_resource {
        return Err(ReflectError::AddressKindMismatch {
            expected: format!("resource `{}`", resource.type_path),
            actual: format!("non-resource `{}`", resource.type_path),
        }
        .into());
    }
    let adapter = runtime
        .resource
        .ok_or_else(|| ReflectError::NoResourceAdapter {
            type_path: resource.type_path.clone(),
        })?;
    let already_present = adapter.contains(world);
    let can_create_on_apply = adapter.ensure.is_some();
    if !already_present && !can_create_on_apply {
        return Err(ReflectError::MissingResource {
            type_path: resource.type_path.clone(),
        }
        .into());
    }

    let mut writable_fields = HashMap::with_capacity(runtime.registration.type_info.fields.len());
    for field in &runtime.registration.type_info.fields {
        writable_fields.insert(field.name.as_str(), field.serializable && field.editable);
    }
    let mut writes = Vec::with_capacity(resource.fields.len());
    for field in &resource.fields {
        let Some(writable) = writable_fields.get(field.field_name.as_str()).copied() else {
            return Err(ReflectError::UnknownField {
                type_path: resource.type_path.clone(),
                field_name: field.field_name.clone(),
            }
            .into());
        };
        if writable {
            writes.push((
                field.field_name.clone(),
                remap_reflected_value(&field.value, remap)?,
            ));
        }
    }
    Ok((
        CompiledResourceWrite {
            type_path: resource.type_path.clone(),
            adapter,
            writes,
        },
        ScenePatchPreviewResource {
            type_path: resource.type_path.clone(),
            already_present,
            can_create_on_apply,
            field_count: resource.fields.len(),
        },
    ))
}

fn build_entity_remap(
    scene: &DynamicScene,
    world: &World,
) -> Result<EntityRemap, DynamicSceneError> {
    let mut remap = EntityRemap::new();
    let mut reserved = BTreeSet::new();
    for entity in &scene.entities {
        let target = first_available_entity_id(world, &reserved, entity.source_entity)?;
        reserved.insert(target);
        remap.insert(entity.source_entity, target);
    }
    Ok(remap)
}

fn compile_entity_records(
    scene: &DynamicScene,
    world: &World,
    remap: &EntityRemap,
) -> Result<Vec<NodeRecord>, DynamicSceneError> {
    let mut records = Vec::with_capacity(scene.entities.len());
    for entity in &scene.entities {
        let mut record = entity.record.clone();
        let target = remapped_entity(remap, entity.source_entity)?;
        record.id = target;
        record.parent = remapped_parent(world, remap, entity.source_entity, record.parent)?;
        remap_record_entity_references(&mut record, remap);
        records.push(record);
    }
    Ok(records)
}

fn insert_entity_records(
    world: &mut World,
    records: Vec<NodeRecord>,
) -> Result<(), DynamicSceneError> {
    world.insert_owned_node_records(records).map_err(Into::into)
}

fn apply_component_writes(
    world: &mut World,
    writes: Vec<CompiledComponentWrite>,
) -> Result<(), DynamicSceneError> {
    for write in writes {
        match write.kind {
            CompiledComponentWriteKind::Plugin { type_path, value } => {
                world.set_dynamic_component(write.entity, type_path, value)?;
            }
            CompiledComponentWriteKind::Reflected { adapter, writes } => {
                if !writes.is_empty() {
                    adapter.write_fields_by_slot(world, write.entity, writes)?;
                }
            }
        }
    }
    Ok(())
}

fn remapped_entity(
    remap: &EntityRemap,
    source_entity: EntityId,
) -> Result<EntityId, DynamicSceneError> {
    remap
        .get(source_entity)
        .ok_or(DynamicSceneError::CompiledPlanMissingEntityRemap { source_entity })
}

fn apply_resource_writes(
    world: &mut World,
    writes: Vec<CompiledResourceWrite>,
) -> Result<(), DynamicSceneError> {
    for write in writes {
        ensure_reflected_resource_exists(world, &write.type_path, write.adapter)?;
        for (field_name, value) in write.writes {
            write.adapter.write_field(world, &field_name, value)?;
        }
    }
    Ok(())
}

fn first_available_entity_id(
    world: &World,
    reserved: &BTreeSet<EntityId>,
    source: EntityId,
) -> Result<EntityId, DynamicSceneError> {
    let mut candidate = source;
    loop {
        if !world.contains_entity(candidate) && !reserved.contains(&candidate) {
            return Ok(candidate);
        }
        candidate = candidate
            .checked_add(1)
            .ok_or(DynamicSceneError::EntityIdSpaceExhausted {
                source_entity: source,
            })?;
    }
}

fn remapped_parent(
    world: &World,
    remap: &EntityRemap,
    entity: EntityId,
    parent: Option<EntityId>,
) -> Result<Option<EntityId>, DynamicSceneError> {
    let Some(parent) = parent else {
        return Ok(None);
    };
    if let Some(parent) = remap.get(parent) {
        return Ok(Some(parent));
    }
    if world.contains_entity(parent) {
        return Ok(Some(parent));
    }
    Err(DynamicSceneError::MissingSceneParent { entity, parent })
}

fn remap_record_entity_references(record: &mut NodeRecord, remap: &EntityRemap) {
    if let Some(joint) = &mut record.joint {
        if let Some(entity) = joint.connected_entity {
            joint.connected_entity = Some(remap.get(entity).unwrap_or(entity));
        }
        if let Some(binding) = &mut joint.skeleton_binding {
            binding.skeleton_entity = remap
                .get(binding.skeleton_entity)
                .unwrap_or(binding.skeleton_entity);
        }
    }
}

fn ensure_reflected_resource_exists(
    world: &mut World,
    type_path: &str,
    adapter: crate::scene::reflect::ReflectResource,
) -> Result<(), ReflectError> {
    if adapter.contains(world) {
        return Ok(());
    }
    let _ = adapter.ensure(world)?;
    if adapter.contains(world) {
        return Ok(());
    }
    Err(ReflectError::MissingResource {
        type_path: type_path.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::reflect::{
        ReflectEditorHint, ReflectError, ReflectFieldInfo, ReflectFieldValue,
        ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
        ReflectedValue,
    };

    use crate::scene::{NodeKind, ReflectResource, Resource, World};

    use super::{DynamicScene, apply_compiled_scene_spawn, compile_scene_spawn};

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

        let remap = apply_compiled_scene_spawn(&scene, &mut target, plan)
            .expect("compiled scene should apply");
        assert_eq!(remap.get(source_entity), Some(planned_target));
        assert!(target.contains_entity(planned_target));
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
        let error = apply_compiled_scene_spawn(&scene, &mut target, plan)
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

        let error = apply_compiled_scene_spawn(&scene, &mut target, plan)
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

    #[derive(Debug, PartialEq, Eq)]
    struct RejectingResource(u32);

    impl Resource for RejectingResource {}

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

    const REJECTING_RESOURCE_TYPE_PATH: &str =
        "zircon_runtime::scene::dynamic_scene::scene::spawn::tests::RejectingResource";

    fn register_rejecting_resource(world: &mut World) {
        world
            .type_registry_mut_for_tests()
            .register_resource(
                ReflectTypeRegistration::new(
                    ReflectTypePath::new(REJECTING_RESOURCE_TYPE_PATH, "RejectingResource")
                        .expect("test resource type path should be valid"),
                    "Rejecting Resource",
                    ReflectTypeInfo::struct_with_fields(vec![ReflectFieldInfo::new(
                        "value",
                        "Unsigned",
                        ReflectEditorHint::Unsigned,
                    )]),
                    ReflectSerializationStrategy::ResourceHandle,
                )
                .as_resource()
                .with_remote_visible(true),
                ReflectResource {
                    estimate_stage_clone_bytes: Some(rejecting_resource_stage_clone_bytes),
                    stage_clone: Some(rejecting_resource_stage_clone),
                    ensure: None,
                    contains: rejecting_resource_contains,
                    read_field: rejecting_resource_read_field,
                    read_fields: rejecting_resource_read_fields,
                    write_field: rejecting_resource_write_field,
                },
            )
            .expect("test resource registration should be accepted");
    }

    fn rejecting_resource_stage_clone_bytes(source: &World) -> Result<usize, ReflectError> {
        rejecting_resource(source)?;
        Ok(std::mem::size_of::<RejectingResource>())
    }

    fn rejecting_resource_stage_clone(
        source: &World,
        target: &mut World,
    ) -> Result<(), ReflectError> {
        target.insert_resource(RejectingResource(rejecting_resource(source)?.0));
        Ok(())
    }

    fn rejecting_resource_contains(world: &World) -> bool {
        world.get_resource::<RejectingResource>().is_some()
    }

    fn rejecting_resource_read_field(
        world: &World,
        field_name: &str,
    ) -> Result<ReflectedValue, ReflectError> {
        match field_name {
            "value" => Ok(ReflectedValue::Unsigned(
                rejecting_resource(world)?.0 as u64,
            )),
            _ => Err(ReflectError::UnknownField {
                type_path: REJECTING_RESOURCE_TYPE_PATH.to_string(),
                field_name: field_name.to_string(),
            }),
        }
    }

    fn rejecting_resource_read_fields(
        world: &World,
    ) -> Result<Vec<ReflectFieldValue>, ReflectError> {
        Ok(vec![ReflectFieldValue::new(
            "value",
            rejecting_resource_read_field(world, "value")?,
        )])
    }

    fn rejecting_resource_write_field(
        _world: &mut World,
        field_name: &str,
        _value: ReflectedValue,
    ) -> Result<bool, ReflectError> {
        Err(ReflectError::UnsupportedConversion {
            source: "resource write intentionally rejected by test adapter".to_string(),
            target: format!("{REJECTING_RESOURCE_TYPE_PATH}.{field_name}"),
        })
    }

    fn rejecting_resource(world: &World) -> Result<&RejectingResource, ReflectError> {
        world
            .get_resource::<RejectingResource>()
            .ok_or_else(|| ReflectError::MissingResource {
                type_path: REJECTING_RESOURCE_TYPE_PATH.to_string(),
            })
    }
}
