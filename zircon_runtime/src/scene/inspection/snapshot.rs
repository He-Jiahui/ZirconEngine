use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::reflect::{ReflectFieldInfo, ReflectFieldValue};
use zircon_runtime_interface::world_sync::{EntityRow, QueryFilter, WorldQuery, WorldQueryResult};

use crate::scene::components::{NodeKind, SceneNode};
use crate::scene::reflect::RuntimeTypeRegistration;
use crate::scene::{EntityId, World};

use super::{WorldInspectionField, WorldInspectionHierarchyRow};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldInspection {
    pub generation: u64,
    pub focused_entity: Option<EntityId>,
    pub hierarchy_rows: Vec<WorldInspectionHierarchyRow>,
    pub fields: Vec<WorldInspectionField>,
}

impl WorldInspection {
    pub fn from_world(world: &World, focused: Option<EntityId>) -> Self {
        let focused_entity = focused.filter(|entity| world.contains_entity(*entity));
        let hierarchy_rows = build_hierarchy_rows(world, focused_entity);
        Self {
            generation: world.world_generation(),
            focused_entity,
            hierarchy_rows,
            fields: focused_entity
                .map(|entity| world.inspect_fields(entity))
                .unwrap_or_default(),
        }
    }
}

impl World {
    /// Builds the composed hierarchy-and-field inspection snapshot.
    pub fn inspect_world(&self, focused: Option<EntityId>) -> WorldInspection {
        WorldInspection::from_world(self, focused)
    }

    /// Builds hierarchy rows without coupling the query to editor focus state.
    pub fn inspect_hierarchy(&self) -> Vec<WorldInspectionHierarchyRow> {
        build_hierarchy_rows(self, None)
    }

    /// Builds reflected fields for one existing entity.
    pub fn inspect_fields(&self, entity: EntityId) -> Vec<WorldInspectionField> {
        if !self.contains_entity(entity) {
            return Vec::new();
        }
        build_inspection_fields(self, entity)
    }

    /// Runs one deterministic reflected-component query against the current world generation.
    ///
    /// The query boundary intentionally exposes only editor-visible reflected components. Runtime
    /// storage-only components remain internal until they register an editor reflection contract.
    pub fn query_world(&self, query: &WorldQuery) -> WorldQueryResult {
        let generation = self.world_generation();
        if generation != u64::MAX && query.generation_hint == Some(generation) {
            return WorldQueryResult::NotModified { generation };
        }

        let rows = self
            .node_records()
            .into_iter()
            .filter_map(|node| {
                let fields = self.inspect_fields(node.id);
                query_matches_reflected_components(&fields, &query.filter).then(|| EntityRow {
                    entity: node.id,
                    components: selected_reflected_components(&fields, query),
                })
            })
            .collect();
        query.result_for_generation(generation, rows)
    }
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
    query: &WorldQuery,
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

fn build_hierarchy_rows(
    world: &World,
    focused_entity: Option<EntityId>,
) -> Vec<WorldInspectionHierarchyRow> {
    let nodes = world.node_records();
    build_hierarchy_rows_from_nodes(world, &nodes, focused_entity)
}

pub(super) fn build_hierarchy_rows_from_nodes(
    world: &World,
    nodes: &[SceneNode],
    focused_entity: Option<EntityId>,
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
            focused_entity,
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
                focused_entity,
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
    focused_entity: Option<EntityId>,
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
            focused: focused_entity == Some(pending.entity),
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
    if !metadata.is_component || !metadata.editor_visible {
        return Ok(Vec::new());
    }
    let Some(adapter) = &runtime.component else {
        return Ok(Vec::new());
    };
    if !adapter.contains(world, entity) {
        return Ok(Vec::new());
    }

    let values = adapter.read_fields(world, entity)?;
    let values_by_name = values
        .iter()
        .map(|field| (field.field_name.as_str(), field))
        .collect::<HashMap<_, _>>();

    Ok(metadata
        .type_info
        .fields
        .iter()
        .filter(|field| field.editor_visible)
        .filter_map(|field| {
            values_by_name.get(field.name.as_str()).map(|value| {
                inspection_field_from_reflection(runtime, field, value, metadata.plugin_owned)
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
        component_type_path: metadata.type_path.type_path.clone(),
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
