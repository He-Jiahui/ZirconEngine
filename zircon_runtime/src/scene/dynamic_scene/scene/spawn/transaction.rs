use std::collections::{BTreeSet, HashMap};

use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

use crate::scene::components::NodeRecord;
use crate::scene::reflect::validate_reflected_field_values;
use crate::scene::{ecs::ChangeTick, EntityId, World};

use super::super::DynamicScene;
use crate::scene::dynamic_scene::value::{
    descriptor_fields_to_json_object, reflected_fields_to_json_object,
};
use crate::scene::dynamic_scene::{
    DynamicComponent, DynamicSceneError, EntityRemap, ScenePatchPreviewReport,
};

use super::commit::commit_preflighted_scene_mutation;
use super::preflight_mutation::extract_preflighted_scene_mutation;
use super::preview::build_preview_report;
use super::resource::{
    apply_resource_writes_to_preflight, compile_reflected_writes, compile_resource_writes,
    stage_compiled_resource_writes_bounded, CompiledResourceWrite,
};

// Building the successor cache amortizes at 16 remaps in the dense-collision fixture.
const ENTITY_REMAP_SUCCESSOR_PROBE_MIN_ENTITIES: usize = 16;

/// Owns the target-bound resolution work so apply never reparses a scene payload.
pub(crate) struct CompiledSceneSpawn {
    target: CompiledSceneSpawnTarget,
    pub(super) remap: EntityRemap,
    pub(super) records: Vec<NodeRecord>,
    component_writes: Vec<CompiledComponentWrite>,
    pub(super) resource_writes: Vec<CompiledResourceWrite>,
    component_type_descriptors: Vec<crate::core::framework::scene::ComponentTypeDescriptor>,
    preview: ScenePatchPreviewReport,
}

/// Generation tokens shared by the full preflight plan and its compact commit
/// artifact. The target World is never retained by either phase.
#[derive(Clone, Copy)]
pub(super) struct CompiledSceneSpawnTarget {
    expected_target_generation: u64,
    expected_schema_catalog_generation: u64,
    expected_component_registry_generation: u64,
    expected_target_change_tick: ChangeTick,
}

/// The only part of a compiled spawn permitted to cross the preflight-to-
/// publication boundary. Adapter calls, reflected values, and preview data are
/// consumed before this artifact is created.
pub(super) struct PreparedSceneSpawnCommit {
    pub(super) target: CompiledSceneSpawnTarget,
    pub(super) remap: EntityRemap,
    pub(super) records: Vec<NodeRecord>,
    pub(super) component_type_descriptors:
        Vec<crate::core::framework::scene::ComponentTypeDescriptor>,
}

struct CompiledComponentWrite {
    entity: EntityId,
    kind: CompiledComponentWriteKind,
}

impl CompiledComponentWrite {
    fn type_path(&self) -> &str {
        match &self.kind {
            CompiledComponentWriteKind::Plugin { type_path, .. } => type_path,
            CompiledComponentWriteKind::Reflected { adapter, .. } => &adapter.type_path,
        }
    }

    fn plugin_type_path(&self) -> Option<&str> {
        match &self.kind {
            CompiledComponentWriteKind::Plugin { type_path, .. } => Some(type_path),
            CompiledComponentWriteKind::Reflected { .. } => None,
        }
    }
}

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

impl CompiledSceneSpawn {
    fn into_preview(self) -> ScenePatchPreviewReport {
        self.preview
    }

    fn into_preflight_parts(
        self,
    ) -> (
        PreparedSceneSpawnCommit,
        Vec<CompiledComponentWrite>,
        Vec<CompiledResourceWrite>,
    ) {
        let Self {
            target,
            remap,
            records,
            component_writes,
            resource_writes,
            component_type_descriptors,
            preview: _,
        } = self;
        (
            PreparedSceneSpawnCommit {
                target,
                remap,
                records,
                component_type_descriptors,
            },
            component_writes,
            resource_writes,
        )
    }
}

pub(in crate::scene::dynamic_scene::scene) fn spawn_scene_into(
    scene: &DynamicScene,
    world: &mut World,
) -> Result<EntityRemap, DynamicSceneError> {
    crate::profile_scope!("runtime", "dynamic_scene.transaction", "spawn");
    let plan = compile_scene_spawn(scene, world)?;
    apply_compiled_scene_spawn(world, plan)
}

pub(crate) fn compile_scene_spawn(
    scene: &DynamicScene,
    world: &World,
) -> Result<CompiledSceneSpawn, DynamicSceneError> {
    crate::profile_scope!("runtime", "dynamic_scene.transaction", "compile");
    scene.ensure_supported()?;
    ensure_component_type_descriptors_are_compatible(scene, world)?;
    let remap = build_entity_remap(scene, world)?;
    let records = compile_entity_records(scene, world, &remap)?;
    world.validate_owned_node_records(&records)?;
    let component_writes = compile_component_writes(scene, world, &remap)?;
    let (resource_writes, resources) = compile_resource_writes(scene, world, &remap)?;
    let preview = build_preview_report(scene, world, &remap, resources);
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.compile.entities",
        records.len()
    );
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.compile.component_write_batches",
        component_writes.len()
    );
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.compile.resource_write_batches",
        resource_writes.len()
    );
    Ok(CompiledSceneSpawn {
        target: CompiledSceneSpawnTarget {
            expected_target_generation: world.world_generation(),
            expected_schema_catalog_generation: world.type_registry().schema_catalog_generation(),
            expected_component_registry_generation: world.component_registry_generation(),
            expected_target_change_tick: world.read_change_tick(),
        },
        remap,
        records,
        component_writes,
        resource_writes,
        component_type_descriptors: scene.component_types.clone(),
        preview,
    })
}

pub(crate) fn apply_compiled_scene_spawn(
    world: &mut World,
    plan: CompiledSceneSpawn,
) -> Result<EntityRemap, DynamicSceneError> {
    crate::profile_scope!("runtime", "dynamic_scene.transaction", "apply");
    ensure_compiled_spawn_target_is_current(world, &plan.target)?;
    let (mut preflight, _) = capture_compiled_scene_spawn_preflight(world, &plan, usize::MAX)?;
    let mutation = validate_compiled_scene_spawn_preflight(&mut preflight, plan)?;
    commit_preflighted_scene_mutation(world, mutation)
}

pub(crate) fn commit_preflighted_compiled_scene_spawn(
    world: &mut World,
    mutation: super::preflight_mutation::PreflightedSceneMutation,
) -> Result<EntityRemap, DynamicSceneError> {
    commit_preflighted_scene_mutation(world, mutation)
}

pub(super) fn ensure_compiled_spawn_target_is_current(
    world: &World,
    target: &CompiledSceneSpawnTarget,
) -> Result<(), DynamicSceneError> {
    let actual_schema_generation = world.type_registry().schema_catalog_generation();
    if actual_schema_generation != target.expected_schema_catalog_generation {
        return Err(DynamicSceneError::TargetSchemaChanged {
            expected_generation: target.expected_schema_catalog_generation,
            actual_generation: actual_schema_generation,
        });
    }
    let actual_component_registry_generation = world.component_registry_generation();
    if actual_component_registry_generation != target.expected_component_registry_generation {
        return Err(DynamicSceneError::TargetComponentRegistryChanged {
            expected_generation: target.expected_component_registry_generation,
            actual_generation: actual_component_registry_generation,
        });
    }
    let actual_generation = world.world_generation();
    if actual_generation != target.expected_target_generation {
        return Err(DynamicSceneError::TargetWorldChanged {
            expected_generation: target.expected_target_generation,
            actual_generation,
        });
    }
    let actual_tick = world.read_change_tick();
    if actual_tick != target.expected_target_change_tick {
        return Err(DynamicSceneError::TargetChangeTickChanged {
            expected_tick: target.expected_target_change_tick.get(),
            actual_tick: actual_tick.get(),
        });
    }
    Ok(())
}

/// Captures the bounded, isolated World needed to validate a compiled scene
/// mutation. No target entity or component storage is cloned or modified.
pub(crate) fn capture_compiled_scene_spawn_preflight(
    target: &World,
    plan: &CompiledSceneSpawn,
    limit_bytes: usize,
) -> Result<(World, usize), DynamicSceneError> {
    crate::profile_scope!("runtime", "dynamic_scene.transaction", "preflight_stage");
    if !target.dynamic_component_type_catalog_is_empty() {
        let declared_type_paths = plan
            .component_type_descriptors
            .iter()
            .map(|descriptor| descriptor.type_id.as_str())
            .collect::<BTreeSet<_>>();
        for type_path in plan
            .component_writes
            .iter()
            .filter_map(CompiledComponentWrite::plugin_type_path)
        {
            if target.component_type_descriptor(type_path).is_none()
                && !declared_type_paths.contains(type_path)
            {
                return Err(DynamicSceneError::WorldMutation(
                    crate::scene::SceneError::UnregisteredDynamicComponentType {
                        component_id: type_path.to_string(),
                    },
                ));
            }
        }
    }
    let affected_type_paths = plan
        .component_type_descriptors
        .iter()
        .map(|descriptor| descriptor.type_id.as_str())
        .chain(
            plan.component_writes
                .iter()
                .map(CompiledComponentWrite::type_path),
        );
    let mut preflight = target.dynamic_scene_preflight_world(affected_type_paths);
    insert_preflight_parent_context(target, &mut preflight, &plan.records)?;
    let base_estimated_bytes = preflight.estimate_dynamic_scene_preflight_bytes(limit_bytes)?;
    let estimated_bytes = stage_compiled_resource_writes_bounded(
        target,
        &mut preflight,
        &plan.resource_writes,
        base_estimated_bytes,
        limit_bytes,
    )?;
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.preflight_staged_bytes",
        estimated_bytes
    );
    Ok((preflight, estimated_bytes))
}

/// Runs every fallible reflected write against the captured preflight World.
/// The target mutation can consume the same compiled plan only after this
/// returns successfully.
pub(crate) fn validate_compiled_scene_spawn_preflight(
    preflight: &mut World,
    plan: CompiledSceneSpawn,
) -> Result<super::preflight_mutation::PreflightedSceneMutation, DynamicSceneError> {
    crate::profile_scope!("runtime", "dynamic_scene.transaction", "preflight_apply");
    let (commit, component_writes, resource_writes) = plan.into_preflight_parts();
    let component_write_batches = component_writes.len();
    let resource_write_batches = resource_writes.len();
    let resource_adapters = resource_writes
        .iter()
        .map(|write| write.adapter)
        .collect::<Vec<_>>();
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.preflight.component_write_batches",
        component_write_batches
    );
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.preflight.resource_write_batches",
        resource_write_batches
    );
    apply_compiled_scene_spawn_unchecked(preflight, &commit, component_writes, resource_writes)?;
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.commit_artifact.released_component_write_batches",
        component_write_batches
    );
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.commit_artifact.released_resource_write_batches",
        resource_write_batches
    );
    extract_preflighted_scene_mutation(preflight, commit, &resource_adapters)
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
    world: &mut World,
    commit: &PreparedSceneSpawnCommit,
    component_writes: Vec<CompiledComponentWrite>,
    resource_writes: Vec<CompiledResourceWrite>,
) -> Result<(), DynamicSceneError> {
    crate::profile_scope!("runtime", "dynamic_scene.transaction", "apply_unchecked");
    {
        crate::profile_scope!(
            "runtime",
            "dynamic_scene.transaction",
            "install_descriptors"
        );
        install_component_type_descriptors(&commit.component_type_descriptors, world)?;
    }
    {
        crate::profile_scope!("runtime", "dynamic_scene.transaction", "insert_records");
        insert_entity_records(world, commit.records.clone())?;
    }
    {
        crate::profile_scope!("runtime", "dynamic_scene.transaction", "apply_components");
        apply_component_writes(world, component_writes)?;
    }
    {
        crate::profile_scope!("runtime", "dynamic_scene.transaction", "apply_resources");
        apply_resource_writes_to_preflight(world, resource_writes)?;
    }
    Ok(())
}

pub(in crate::scene::dynamic_scene::scene) fn preview_scene_spawn_into(
    scene: &DynamicScene,
    world: &World,
) -> Result<ScenePatchPreviewReport, DynamicSceneError> {
    Ok(compile_scene_spawn(scene, world)?.into_preview())
}

pub(super) fn install_component_type_descriptors(
    descriptors: &[crate::core::framework::scene::ComponentTypeDescriptor],
    world: &mut World,
) -> Result<(), DynamicSceneError> {
    for descriptor in descriptors {
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
    let descriptors_by_type = scene
        .component_types
        .iter()
        .map(|descriptor| (descriptor.type_id.as_str(), descriptor))
        .collect::<HashMap<_, _>>();
    let mut writes = Vec::with_capacity(component_count);
    for entity in &scene.entities {
        let target = remapped_entity(remap, entity.source_entity)?;
        for component in &entity.components {
            writes.push(compile_component_write(
                world,
                target,
                component,
                descriptors_by_type
                    .get(component.type_path.as_str())
                    .copied(),
                remap,
            )?);
        }
    }
    Ok(writes)
}

fn compile_component_write(
    world: &World,
    entity: EntityId,
    component: &DynamicComponent,
    descriptor: Option<&crate::core::framework::scene::ComponentTypeDescriptor>,
    remap: &EntityRemap,
) -> Result<CompiledComponentWrite, DynamicSceneError> {
    validate_reflected_field_values(&component.type_path, &component.fields)?;
    if component.plugin_owned {
        let resolved_type_path = world
            .type_registry()
            .resolve(&component.type_path)
            .unwrap_or(&component.type_path)
            .to_string();
        let value = match world
            .type_registry()
            .runtime_registration(&component.type_path)
        {
            Ok(runtime) => reflected_fields_to_json_object(
                world,
                &component.type_path,
                &runtime.registration.type_info.fields,
                &component.fields,
                remap,
            )?,
            Err(ReflectError::UnknownType { .. }) => {
                let descriptor = descriptor.ok_or_else(|| ReflectError::UnknownType {
                    type_path: component.type_path.clone(),
                })?;
                descriptor_fields_to_json_object(descriptor, &component.fields, remap)?
            }
            Err(error) => return Err(error.into()),
        };
        return Ok(CompiledComponentWrite {
            entity,
            kind: CompiledComponentWriteKind::Plugin {
                type_path: resolved_type_path,
                value,
            },
        });
    }

    let runtime = world
        .type_registry()
        .runtime_registration(&component.type_path)?;
    if !runtime.registration.is_component() {
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

    let writes = compile_reflected_writes(
        world,
        &component.type_path,
        &runtime.registration.type_info.fields,
        &component.fields,
        remap,
    )?;
    Ok(CompiledComponentWrite {
        entity,
        kind: CompiledComponentWriteKind::Reflected { adapter, writes },
    })
}

fn build_entity_remap(
    scene: &DynamicScene,
    world: &World,
) -> Result<EntityRemap, DynamicSceneError> {
    if scene.entities.len() < ENTITY_REMAP_SUCCESSOR_PROBE_MIN_ENTITIES {
        return build_entity_remap_linear(scene, world);
    }

    let mut remap = EntityRemap::new();
    let mut probe = EntityIdReservationProbe::new(world);
    for entity in &scene.entities {
        let target = probe.reserve(entity.source_entity)?;
        remap.insert(entity.source_entity, target);
    }
    Ok(remap)
}

struct EntityIdReservationProbe<'world> {
    world: &'world World,
    successor_by_occupied: HashMap<EntityId, Option<EntityId>>,
    path: Vec<EntityId>,
}

impl<'world> EntityIdReservationProbe<'world> {
    fn new(world: &'world World) -> Self {
        Self {
            world,
            successor_by_occupied: HashMap::new(),
            path: Vec::new(),
        }
    }

    fn reserve(&mut self, source: EntityId) -> Result<EntityId, DynamicSceneError> {
        self.path.clear();
        let mut candidate = source;
        loop {
            if let Some(successor) = self.successor_by_occupied.get(&candidate).copied() {
                self.path.push(candidate);
                candidate = successor.ok_or(DynamicSceneError::EntityIdSpaceExhausted {
                    source_entity: source,
                })?;
                continue;
            }

            let successor = candidate.checked_add(1);
            if self.world.contains_entity(candidate) {
                self.path.push(candidate);
                self.successor_by_occupied.insert(candidate, successor);
                candidate = successor.ok_or(DynamicSceneError::EntityIdSpaceExhausted {
                    source_entity: source,
                })?;
                continue;
            }

            self.successor_by_occupied.insert(candidate, successor);
            for skipped in self.path.drain(..) {
                self.successor_by_occupied.insert(skipped, successor);
            }
            return Ok(candidate);
        }
    }
}

fn build_entity_remap_linear(
    scene: &DynamicScene,
    world: &World,
) -> Result<EntityRemap, DynamicSceneError> {
    let mut remap = EntityRemap::new();
    let mut reserved = BTreeSet::new();
    for entity in &scene.entities {
        let mut candidate = entity.source_entity;
        loop {
            if !world.contains_entity(candidate) && !reserved.contains(&candidate) {
                reserved.insert(candidate);
                remap.insert(entity.source_entity, candidate);
                break;
            }
            candidate =
                candidate
                    .checked_add(1)
                    .ok_or(DynamicSceneError::EntityIdSpaceExhausted {
                        source_entity: entity.source_entity,
                    })?;
        }
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
    crate::profile_scope!(
        "runtime",
        "dynamic_scene.transaction",
        "component_adapter_apply"
    );
    let mut plugin_component_writes = 0_usize;
    let mut reflected_adapter_write_calls = 0_usize;
    let mut reflected_field_writes = 0_usize;
    for write in &writes {
        match &write.kind {
            CompiledComponentWriteKind::Plugin { .. } => plugin_component_writes += 1,
            CompiledComponentWriteKind::Reflected { writes, .. } => {
                reflected_adapter_write_calls += usize::from(!writes.is_empty());
                reflected_field_writes = reflected_field_writes.saturating_add(writes.len());
            }
        }
    }
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.component_adapter.plugin_component_writes",
        plugin_component_writes
    );
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.component_adapter.write_fields_calls",
        reflected_adapter_write_calls
    );
    crate::profile_counter!(
        "runtime",
        "dynamic_scene.transaction.component_adapter.field_writes",
        reflected_field_writes
    );
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

#[cfg(test)]
#[path = "transaction/performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "transaction/tests.rs"]
mod tests;
