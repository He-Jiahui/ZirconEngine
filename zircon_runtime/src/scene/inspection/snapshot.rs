use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{self, Write};
use std::time::{Duration, Instant};

use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Serialize, Serializer};
use zircon_runtime_interface::reflect::{ReflectFieldInfo, ReflectFieldValue, ReflectedValue};
use zircon_runtime_interface::world_sync::{
    ComponentWorldQuery, EntityRow, QueryFilter, WorldQuery, WorldQueryResult,
};
use zircon_runtime_interface::ZrRuntimePayloadLimitV1;

use crate::scene::components::{NodeKind, SceneNode};
use crate::scene::reflect::{
    validate_reflected_field_values, validate_reflected_value, RuntimeTypeRegistration,
};
use crate::scene::{EntityId, World, WorldQueryBudgetError};

use super::{WorldInspectionField, WorldInspectionHierarchyRow};

impl World {
    /// Runs one deterministic reflected-component query against the current world generation.
    ///
    /// The query boundary intentionally exposes only editor-visible reflected components. Runtime
    /// storage-only components remain internal until they register an editor reflection contract.
    pub fn query_world(&self, query: &WorldQuery) -> WorldQueryResult {
        self.query_world_at_replacement_epoch(query, 1)
    }

    pub(crate) fn query_world_at_replacement_epoch(
        &self,
        query: &WorldQuery,
        world_replacement_epoch: u64,
    ) -> WorldQueryResult {
        let generation = self.world_generation();
        if generation != u64::MAX && query.generation_hint() == Some(generation) {
            return WorldQueryResult::NotModified { generation };
        }

        match query {
            WorldQuery::Components(component_query) => {
                let rows = self
                    .node_records()
                    .into_iter()
                    .filter_map(|node| {
                        let fields = build_inspection_fields(self, node.id);
                        query_matches_reflected_components(&fields, &component_query.filter).then(
                            || EntityRow {
                                entity: node.id,
                                components: selected_reflected_components(&fields, component_query),
                            },
                        )
                    })
                    .collect();
                query.component_result_for_generation(generation, rows)
            }
            WorldQuery::Hierarchy(_) => {
                let artifact = self.inspection_artifact();
                query.hierarchy_result_for_generation(
                    artifact.generation(),
                    artifact.hierarchy_rows().to_vec(),
                )
            }
            WorldQuery::InspectionFields(fields_query) => {
                let artifact = self.inspection_fields_artifact(fields_query.entity);
                let generation = artifact
                    .as_ref()
                    .map(|artifact| artifact.generation())
                    .unwrap_or(generation);
                query.inspection_fields_result_for_generation(
                    generation,
                    fields_query.entity,
                    artifact.map(|artifact| artifact.fields().to_vec()),
                )
            }
            WorldQuery::TransformSnapshot(transform_query) => query.transform_snapshot_result(
                generation,
                world_replacement_epoch,
                transform_query.entity,
                self.local_transform(transform_query.entity),
            ),
        }
    }

    pub(crate) fn query_world_bounded(
        &self,
        query: &WorldQuery,
        limit: ZrRuntimePayloadLimitV1,
    ) -> Result<WorldQueryResult, WorldQueryBudgetError> {
        self.query_world_bounded_at_replacement_epoch(query, limit, 1)
    }

    pub(crate) fn query_world_bounded_at_replacement_epoch(
        &self,
        query: &WorldQuery,
        limit: ZrRuntimePayloadLimitV1,
        world_replacement_epoch: u64,
    ) -> Result<WorldQueryResult, WorldQueryBudgetError> {
        let mut budget = WorldQueryBuildBudget::new(limit);
        let generation = self.world_generation();
        if generation != u64::MAX && query.generation_hint() == Some(generation) {
            let result = WorldQueryResult::NotModified { generation };
            budget.validate_result(&result)?;
            return Ok(result);
        }

        match query {
            WorldQuery::Components(component_query) => {
                let mut rows = Vec::new();
                budget.observe_rows_candidate(generation, &rows, None)?;
                budget.check_deadline()?;
                for entity in self.entity_ids_for_query() {
                    budget.check_deadline()?;
                    if !query_matches_world_components(self, entity, &component_query.filter) {
                        continue;
                    }
                    budget.ensure_additional_items(1)?;
                    let (components, component_items) = selected_reflected_components_bounded(
                        self,
                        entity,
                        component_query,
                        &mut budget,
                    )?;
                    let row_items = component_items.saturating_add(1);
                    budget.observe_items(row_items)?;
                    let row = EntityRow { entity, components };
                    budget.observe_rows_candidate(generation, &rows, Some(&row))?;
                    rows.push(row);
                }
                budget.check_deadline()?;
                Ok(query.component_result_for_generation(generation, rows))
            }
            WorldQuery::Hierarchy(_) => {
                let artifact = self.inspection_artifact();
                let hierarchy_rows = artifact.hierarchy_rows();
                budget.observe_items(hierarchy_rows.len())?;
                budget.validate_payload(&WorldQueryHierarchyRowsCandidate {
                    generation: artifact.generation(),
                    rows: hierarchy_rows,
                })?;
                let rows = hierarchy_rows.to_vec();
                budget.check_deadline()?;
                Ok(query.hierarchy_result_for_generation(artifact.generation(), rows))
            }
            WorldQuery::InspectionFields(fields_query) => {
                let Some(artifact) = self.inspection_fields_artifact(fields_query.entity) else {
                    let result = WorldQueryResult::EntityMissing {
                        generation,
                        entity: fields_query.entity,
                    };
                    budget.validate_result(&result)?;
                    return Ok(result);
                };
                let fields = artifact.fields();
                budget.observe_items(fields.len())?;
                budget.validate_payload(&WorldQueryInspectionFieldsCandidate {
                    generation: artifact.generation(),
                    entity: fields_query.entity,
                    fields,
                })?;
                let fields = fields.to_vec();
                budget.check_deadline()?;
                Ok(query.inspection_fields_result_for_generation(
                    artifact.generation(),
                    fields_query.entity,
                    Some(fields),
                ))
            }
            WorldQuery::TransformSnapshot(transform_query) => {
                budget.observe_items(1)?;
                let result = query.transform_snapshot_result(
                    generation,
                    world_replacement_epoch,
                    transform_query.entity,
                    self.local_transform(transform_query.entity),
                );
                budget.validate_result(&result)?;
                Ok(result)
            }
        }
    }
}

struct WorldQueryBuildBudget {
    limit: ZrRuntimePayloadLimitV1,
    started: Instant,
    item_count: usize,
    encoded_payload_bytes: usize,
    preflight_value_bytes: usize,
}

impl WorldQueryBuildBudget {
    fn new(limit: ZrRuntimePayloadLimitV1) -> Self {
        Self {
            limit,
            started: Instant::now(),
            item_count: 0,
            encoded_payload_bytes: 0,
            preflight_value_bytes: 0,
        }
    }

    fn check_deadline(&self) -> Result<(), WorldQueryBudgetError> {
        if self.started.elapsed() > Duration::from_micros(self.limit.max_processing_time_micros) {
            return Err(WorldQueryBudgetError::ProcessingTime {
                limit_micros: self.limit.max_processing_time_micros,
            });
        }
        Ok(())
    }

    fn observe_items(&mut self, count: usize) -> Result<(), WorldQueryBudgetError> {
        self.item_count = self.item_count.saturating_add(count);
        if self.item_count > self.limit.max_items {
            return Err(WorldQueryBudgetError::Items {
                observed: self.item_count,
                limit: self.limit.max_items,
            });
        }
        Ok(())
    }

    fn ensure_additional_items(&self, count: usize) -> Result<(), WorldQueryBudgetError> {
        let observed = self.item_count.saturating_add(count);
        if observed > self.limit.max_items {
            return Err(WorldQueryBudgetError::Items {
                observed,
                limit: self.limit.max_items,
            });
        }
        Ok(())
    }

    fn preflight_reflected_value(
        &mut self,
        value: &ReflectedValue,
    ) -> Result<(), WorldQueryBudgetError> {
        self.check_deadline()?;
        self.ensure_nesting_offset(5)?;
        let mut writer = WorldQueryCountingWriter::new(
            self.encoded_payload_bytes
                .saturating_add(self.preflight_value_bytes),
            self.limit.max_encoded_bytes,
            self.limit.max_nesting_depth,
            5,
            self.started,
            self.limit.max_processing_time_micros,
        );
        let result = serde_json::to_writer(&mut writer, value);
        let value_bytes = writer.finish_with_count(result)?;
        self.preflight_value_bytes = self.preflight_value_bytes.saturating_add(value_bytes);
        Ok(())
    }

    fn observe_rows_candidate(
        &mut self,
        generation: u64,
        rows: &[EntityRow],
        next: Option<&EntityRow>,
    ) -> Result<(), WorldQueryBudgetError> {
        self.check_deadline()?;
        let mut writer = WorldQueryCountingWriter::new(
            0,
            self.limit.max_encoded_bytes,
            self.limit.max_nesting_depth,
            0,
            self.started,
            self.limit.max_processing_time_micros,
        );
        let candidate = WorldQueryRowsCandidate {
            generation,
            rows,
            next,
        };
        let result = serde_json::to_writer(&mut writer, &candidate);
        self.encoded_payload_bytes = writer.finish_with_count(result)?;
        self.preflight_value_bytes = 0;
        Ok(())
    }

    fn validate_result(&mut self, result: &WorldQueryResult) -> Result<(), WorldQueryBudgetError> {
        self.validate_payload(result)
    }

    fn validate_payload(&mut self, payload: &impl Serialize) -> Result<(), WorldQueryBudgetError> {
        self.check_deadline()?;
        let mut writer = WorldQueryCountingWriter::new(
            0,
            self.limit.max_encoded_bytes,
            self.limit.max_nesting_depth,
            0,
            self.started,
            self.limit.max_processing_time_micros,
        );
        let result = serde_json::to_writer(&mut writer, payload);
        self.encoded_payload_bytes = writer.finish_with_count(result)?;
        Ok(())
    }

    fn ensure_nesting_offset(&self, offset: usize) -> Result<(), WorldQueryBudgetError> {
        if offset > self.limit.max_nesting_depth {
            return Err(WorldQueryBudgetError::NestingDepth {
                observed: offset,
                limit: self.limit.max_nesting_depth,
            });
        }
        Ok(())
    }
}

struct WorldQueryHierarchyRowsCandidate<'a> {
    generation: u64,
    rows: &'a [WorldInspectionHierarchyRow],
}

struct WorldQueryInspectionFieldsCandidate<'a> {
    generation: u64,
    entity: EntityId,
    fields: &'a [WorldInspectionField],
}

impl Serialize for WorldQueryInspectionFieldsCandidate<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("WorldQueryResult", 2)?;
        state.serialize_field("kind", "inspection_fields")?;
        state.serialize_field(
            "data",
            &WorldQueryInspectionFieldsData {
                generation: self.generation,
                entity: self.entity,
                fields: self.fields,
            },
        )?;
        state.end()
    }
}

#[derive(Serialize)]
struct WorldQueryInspectionFieldsData<'a> {
    generation: u64,
    entity: EntityId,
    fields: &'a [WorldInspectionField],
}

impl Serialize for WorldQueryHierarchyRowsCandidate<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("WorldQueryResult", 2)?;
        state.serialize_field("kind", "hierarchy_rows")?;
        state.serialize_field(
            "data",
            &WorldQueryHierarchyRowsData {
                generation: self.generation,
                rows: self.rows,
            },
        )?;
        state.end()
    }
}

#[derive(Serialize)]
struct WorldQueryHierarchyRowsData<'a> {
    generation: u64,
    rows: &'a [WorldInspectionHierarchyRow],
}

struct WorldQueryRowsCandidate<'a> {
    generation: u64,
    rows: &'a [EntityRow],
    next: Option<&'a EntityRow>,
}

impl Serialize for WorldQueryRowsCandidate<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("WorldQueryResult", 2)?;
        state.serialize_field("kind", "component_rows")?;
        state.serialize_field(
            "data",
            &WorldQueryRowsData {
                generation: self.generation,
                rows: self.rows,
                next: self.next,
            },
        )?;
        state.end()
    }
}

struct WorldQueryRowsData<'a> {
    generation: u64,
    rows: &'a [EntityRow],
    next: Option<&'a EntityRow>,
}

impl Serialize for WorldQueryRowsData<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ComponentRows", 2)?;
        state.serialize_field("generation", &self.generation)?;
        state.serialize_field(
            "rows",
            &WorldQueryRowsSequence {
                rows: self.rows,
                next: self.next,
            },
        )?;
        state.end()
    }
}

struct WorldQueryRowsSequence<'a> {
    rows: &'a [EntityRow],
    next: Option<&'a EntityRow>,
}

impl Serialize for WorldQueryRowsSequence<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(
            self.rows
                .len()
                .saturating_add(usize::from(self.next.is_some())),
        ))?;
        for row in self.rows {
            sequence.serialize_element(row)?;
        }
        if let Some(next) = self.next {
            sequence.serialize_element(next)?;
        }
        sequence.end()
    }
}

struct WorldQueryCountingWriter {
    base_count: usize,
    count: usize,
    max_bytes: usize,
    max_depth: usize,
    depth: usize,
    in_string: bool,
    escaped: bool,
    started: Instant,
    max_processing_time_micros: u64,
    failure: Option<WorldQueryBudgetError>,
}

impl WorldQueryCountingWriter {
    fn new(
        base_count: usize,
        max_bytes: usize,
        max_depth: usize,
        nesting_offset: usize,
        started: Instant,
        max_processing_time_micros: u64,
    ) -> Self {
        Self {
            base_count,
            count: 0,
            max_bytes,
            max_depth,
            depth: nesting_offset,
            in_string: false,
            escaped: false,
            started,
            max_processing_time_micros,
            failure: None,
        }
    }

    fn finish_with_count(
        mut self,
        result: Result<(), serde_json::Error>,
    ) -> Result<usize, WorldQueryBudgetError> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        result.map_err(|error| WorldQueryBudgetError::Json(error.to_string()))?;
        if self.started.elapsed() > Duration::from_micros(self.max_processing_time_micros) {
            return Err(WorldQueryBudgetError::ProcessingTime {
                limit_micros: self.max_processing_time_micros,
            });
        }
        Ok(self.count)
    }
}

impl Write for WorldQueryCountingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.started.elapsed() > Duration::from_micros(self.max_processing_time_micros) {
            self.failure = Some(WorldQueryBudgetError::ProcessingTime {
                limit_micros: self.max_processing_time_micros,
            });
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "world query build deadline exceeded",
            ));
        }
        let count = self.count.saturating_add(bytes.len());
        let observed = self.base_count.saturating_add(count);
        if observed > self.max_bytes {
            self.failure = Some(WorldQueryBudgetError::EncodedBytes {
                observed,
                limit: self.max_bytes,
            });
            return Err(io::Error::other("world query byte budget exceeded"));
        }
        for byte in bytes {
            if self.in_string {
                if self.escaped {
                    self.escaped = false;
                } else if *byte == b'\\' {
                    self.escaped = true;
                } else if *byte == b'"' {
                    self.in_string = false;
                }
                continue;
            }
            match *byte {
                b'"' => self.in_string = true,
                b'{' | b'[' => {
                    self.depth = self.depth.saturating_add(1);
                    if self.depth > self.max_depth {
                        self.failure = Some(WorldQueryBudgetError::NestingDepth {
                            observed: self.depth,
                            limit: self.max_depth,
                        });
                        return Err(io::Error::other("world query nesting budget exceeded"));
                    }
                }
                b'}' | b']' => self.depth = self.depth.saturating_sub(1),
                _ => {}
            }
        }
        self.count = count;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn selected_reflected_components_bounded(
    world: &World,
    entity: EntityId,
    query: &ComponentWorldQuery,
    budget: &mut WorldQueryBuildBudget,
) -> Result<(BTreeMap<String, serde_json::Value>, usize), WorldQueryBudgetError> {
    let mut components = BTreeMap::new();
    let mut item_count = 0_usize;
    for selector in &query.select {
        budget.check_deadline()?;
        let Some(runtime) = reflected_component_runtime(world, entity, &selector.type_name) else {
            continue;
        };
        let Some(adapter) = runtime.component.as_ref() else {
            continue;
        };
        let mut values = serde_json::Map::new();
        for field in runtime
            .registration
            .type_info
            .fields
            .iter()
            .filter(|field| field.editor_visible)
        {
            budget.check_deadline()?;
            let Ok(value) = adapter.read_field(world, entity, &field.name) else {
                continue;
            };
            validate_reflected_value(
                runtime.registration.type_path.type_path(),
                &field.name,
                &value,
            )
            .map_err(|error| WorldQueryBudgetError::ReflectValue(error.to_string()))?;
            budget.ensure_additional_items(item_count.saturating_add(2))?;
            budget.preflight_reflected_value(&value)?;
            let value = serde_json::to_value(&value)
                .map_err(|error| WorldQueryBudgetError::Json(error.to_string()))?;
            values.insert(field.name.clone(), value);
            item_count = item_count.saturating_add(1);
        }
        if !values.is_empty() {
            budget.ensure_additional_items(item_count.saturating_add(2))?;
            item_count = item_count.saturating_add(1);
            components.insert(selector.type_name.clone(), values.into());
        }
    }
    Ok((components, item_count))
}

fn reflected_component_runtime<'a>(
    world: &'a World,
    entity: EntityId,
    type_name: &str,
) -> Option<&'a RuntimeTypeRegistration> {
    world.type_registry().iter().find(|runtime| {
        let metadata = &runtime.registration;
        metadata.type_path.type_path() == type_name
            && metadata.is_component()
            && metadata.editor_visible
            && runtime
                .component
                .as_ref()
                .is_some_and(|adapter| adapter.contains(world, entity))
    })
}

fn query_matches_world_components(world: &World, entity: EntityId, filter: &QueryFilter) -> bool {
    filter
        .with
        .iter()
        .all(|type_name| reflected_component_runtime(world, entity, type_name).is_some())
        && filter
            .without
            .iter()
            .all(|type_name| reflected_component_runtime(world, entity, type_name).is_none())
}

fn query_matches_reflected_components(
    fields: &[WorldInspectionField],
    filter: &QueryFilter,
) -> bool {
    filter.with.iter().all(|type_name| {
        fields
            .iter()
            .any(|field| field.component_type_path.as_str() == type_name.as_str())
    }) && filter.without.iter().all(|type_name| {
        !fields
            .iter()
            .any(|field| field.component_type_path.as_str() == type_name.as_str())
    })
}

fn selected_reflected_components(
    fields: &[WorldInspectionField],
    query: &ComponentWorldQuery,
) -> BTreeMap<String, serde_json::Value> {
    query
        .select
        .iter()
        .filter_map(|selector| {
            let values = fields
                .iter()
                .filter(|field| field.component_type_path.as_str() == selector.type_name.as_str())
                .filter_map(|field| {
                    serde_json::to_value(&field.value)
                        .ok()
                        .map(|value| (field.field_name.clone(), value))
                })
                .collect::<serde_json::Map<_, _>>();
            (!values.is_empty()).then(|| (selector.type_name.clone(), values.into()))
        })
        .collect()
}

pub(super) fn build_hierarchy_rows_from_nodes(
    world: &World,
    nodes: &[SceneNode],
) -> Vec<WorldInspectionHierarchyRow> {
    let node_by_entity = nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<HashMap<_, _>>();
    let mut children_by_parent: HashMap<Option<EntityId>, Vec<EntityId>> =
        HashMap::with_capacity(nodes.len());
    for node in nodes {
        children_by_parent
            .entry(node.parent)
            .or_default()
            .push(node.id);
    }

    let mut rows: Vec<WorldInspectionHierarchyRow> = Vec::with_capacity(nodes.len());
    let mut visited = HashSet::with_capacity(nodes.len());
    if let Some(roots) = children_by_parent.get(&None) {
        push_hierarchy_roots(
            world,
            &node_by_entity,
            &children_by_parent,
            roots.iter().copied(),
            &mut visited,
            &mut rows,
        );
    }

    for node in nodes {
        if !visited.contains(&node.id) {
            push_hierarchy_roots(
                world,
                &node_by_entity,
                &children_by_parent,
                std::iter::once(node.id),
                &mut visited,
                &mut rows,
            );
        }
    }
    rows
}

#[derive(Clone, Copy)]
struct PendingHierarchyRow {
    entity: EntityId,
    depth: u32,
    expanded: bool,
    edge_parent: Option<EntityId>,
}

/// Emits pre-order rows and computes their hashes on post-order stack frames,
/// avoiding recursion for editor-scale deep hierarchy chains.
fn push_hierarchy_roots(
    world: &World,
    node_by_entity: &HashMap<EntityId, &SceneNode>,
    children_by_parent: &HashMap<Option<EntityId>, Vec<EntityId>>,
    roots: impl IntoIterator<Item = EntityId>,
    visited: &mut HashSet<EntityId>,
    rows: &mut Vec<WorldInspectionHierarchyRow>,
) {
    let roots = roots.into_iter().collect::<Vec<_>>();
    let mut stack = Vec::new();
    for root in roots.into_iter().rev() {
        stack.push(PendingHierarchyRow {
            entity: root,
            depth: 0,
            expanded: false,
            edge_parent: None,
        });
    }
    let mut row_indices: HashMap<EntityId, usize> = HashMap::new();
    let mut subtree_hashes: HashMap<EntityId, u64> = HashMap::new();
    let mut traversed_edges: HashSet<(EntityId, EntityId)> = HashSet::new();

    while let Some(pending) = stack.pop() {
        if pending.expanded {
            let Some(row_index) = row_indices.get(&pending.entity).copied() else {
                continue;
            };
            let children = children_by_parent.get(&Some(pending.entity));
            let subtree_hash = hierarchy_subtree_hash(
                &rows[row_index].display_name,
                children.map_or(0, Vec::len),
                children.into_iter().flatten().map(|child| {
                    let child_hash = if traversed_edges.contains(&(pending.entity, *child)) {
                        subtree_hashes.get(child).copied().unwrap_or_default()
                    } else {
                        0
                    };
                    (*child, child_hash)
                }),
            );
            rows[row_index].subtree_hash = subtree_hash;
            subtree_hashes.insert(pending.entity, subtree_hash);
            continue;
        }

        if !visited.insert(pending.entity) {
            continue;
        }
        let Some(node) = node_by_entity.get(&pending.entity).copied() else {
            continue;
        };
        if let Some(edge_parent) = pending.edge_parent {
            traversed_edges.insert((edge_parent, pending.entity));
        }
        let children = children_by_parent.get(&Some(pending.entity));
        let row_index = rows.len();
        row_indices.insert(pending.entity, row_index);
        rows.push(WorldInspectionHierarchyRow {
            entity: pending.entity,
            parent: node.parent,
            depth: pending.depth,
            display_name: node.name.clone(),
            kind: node_kind_label(&node.kind).to_string(),
            subtree_hash: 0,
            active_in_hierarchy: world.active_in_hierarchy(pending.entity).unwrap_or(false),
            has_children: children.is_some_and(|children| !children.is_empty()),
        });

        stack.push(PendingHierarchyRow {
            expanded: true,
            ..pending
        });
        if let Some(children) = children {
            for child in children.iter().rev() {
                stack.push(PendingHierarchyRow {
                    entity: *child,
                    depth: pending.depth.saturating_add(1),
                    expanded: false,
                    edge_parent: Some(pending.entity),
                });
            }
        }
    }
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

struct StableSubtreeHasher {
    value: u64,
}

impl StableSubtreeHasher {
    fn new(display_name: &str) -> Self {
        let mut hasher = Self {
            value: FNV_OFFSET_BASIS,
        };
        hasher.write_u64(display_name.len() as u64);
        hasher.write_bytes(display_name.as_bytes());
        hasher
    }

    fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.value ^= u64::from(*byte);
            self.value = self.value.wrapping_mul(FNV_PRIME);
        }
    }

    const fn finish(self) -> u64 {
        self.value
    }
}

/// Hashes one hierarchy anchor from its display name and ordered child anchors.
///
/// The inspection artifact's incremental path reuses this exact representation when a name
/// mutation changes only an existing row and its ancestor hashes.
pub(super) fn hierarchy_subtree_hash(
    display_name: &str,
    child_count: usize,
    children: impl IntoIterator<Item = (EntityId, u64)>,
) -> u64 {
    let child_hash_aggregate =
        children
            .into_iter()
            .enumerate()
            .fold(0, |aggregate, (ordinal, (child, child_hash))| {
                aggregate ^ hierarchy_child_hash_contribution(ordinal, child, child_hash)
            });
    hierarchy_subtree_hash_from_child_aggregate(display_name, child_count, child_hash_aggregate)
}

pub(super) fn hierarchy_subtree_hash_from_child_aggregate(
    display_name: &str,
    child_count: usize,
    child_hash_aggregate: u64,
) -> u64 {
    let mut subtree_hasher = StableSubtreeHasher::new(display_name);
    subtree_hasher.write_u64(child_count as u64);
    subtree_hasher.write_u64(child_hash_aggregate);
    subtree_hasher.finish()
}

pub(super) fn hierarchy_child_hash_contribution(
    ordinal: usize,
    child: EntityId,
    child_hash: u64,
) -> u64 {
    let mut child_hasher = StableSubtreeHasher::new("");
    child_hasher.write_u64(ordinal as u64);
    child_hasher.write_u64(child);
    child_hasher.write_u64(child_hash);
    child_hasher.finish()
}

pub(super) fn build_inspection_fields(
    world: &World,
    entity: EntityId,
) -> Vec<WorldInspectionField> {
    let mut fields = Vec::new();
    for runtime in world.type_registry().iter() {
        if let Ok(component_fields) = reflected_component_fields(world, entity, runtime) {
            fields.extend(component_fields);
        }
    }
    fields.sort_by(|left, right| {
        left.component_display_name
            .cmp(&right.component_display_name)
            .then_with(|| left.field_display_name.cmp(&right.field_display_name))
            .then_with(|| left.component_type_path.cmp(&right.component_type_path))
            .then_with(|| left.field_name.cmp(&right.field_name))
    });
    fields
}

fn reflected_component_fields(
    world: &World,
    entity: EntityId,
    runtime: &RuntimeTypeRegistration,
) -> Result<Vec<WorldInspectionField>, zircon_runtime_interface::reflect::ReflectError> {
    let metadata = &runtime.registration;
    if !metadata.is_component() || !metadata.editor_visible {
        return Ok(Vec::new());
    }
    let Some(adapter) = &runtime.component else {
        return Ok(Vec::new());
    };
    if !adapter.contains(world, entity) {
        return Ok(Vec::new());
    }

    let values = crate::scene::reflect::WorldReflection::read_component_fields_by_slot(
        world, entity, metadata, adapter,
    )?;
    validate_reflected_field_values(metadata.type_path.type_path(), &values)?;
    let values_by_id = values
        .iter()
        .map(|field| (field.field_id, field))
        .collect::<HashMap<_, _>>();

    Ok(metadata
        .type_info
        .fields
        .iter()
        .filter(|field| field.editor_visible)
        .filter_map(|field| {
            values_by_id.get(&field.id).map(|value| {
                inspection_field_from_reflection(runtime, field, value, metadata.is_plugin_owned())
            })
        })
        .collect())
}

fn inspection_field_from_reflection(
    runtime: &RuntimeTypeRegistration,
    field: &ReflectFieldInfo,
    value: &ReflectFieldValue,
    plugin_owned: bool,
) -> WorldInspectionField {
    let metadata = &runtime.registration;
    WorldInspectionField {
        component_type_path: metadata.type_path.type_path().to_string(),
        component_display_name: metadata.display_name.clone(),
        field_name: field.name.clone(),
        field_display_name: field.display_name.clone(),
        value_type_path: field.value_type_path.clone(),
        value: value.value.clone(),
        writable: field.editable,
        serializable: field.serializable,
        plugin_owned,
    }
}

fn node_kind_label(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Empty => "Empty",
        NodeKind::Camera => "Camera",
        NodeKind::Cube => "Cube",
        NodeKind::Mesh => "Mesh",
        NodeKind::AmbientLight => "Ambient Light",
        NodeKind::DirectionalLight => "Directional Light",
        NodeKind::PointLight => "Point Light",
        NodeKind::RectLight => "Rect Light",
        NodeKind::SpotLight => "Spot Light",
    }
}
