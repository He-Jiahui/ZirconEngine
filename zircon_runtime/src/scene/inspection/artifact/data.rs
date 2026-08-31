use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, OnceLock};

use crate::scene::components::SceneNode;
use crate::scene::{EntityId, World};

use super::super::WorldInspectionHierarchyRow;
use super::super::snapshot::{
    build_hierarchy_rows_from_nodes, hierarchy_child_hash_contribution,
    hierarchy_subtree_hash_from_child_aggregate,
};
use super::metrics::HierarchyRowMaterializations;
use super::overrides::{HierarchyChildHashOverrides, HierarchyRowOverrides};

/// Immutable, generation-scoped runtime data shared by scene inspection consumers.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldInspectionArtifact {
    generation: u64,
    hierarchy_rows: Arc<HierarchyRows>,
    row_indices: Arc<HashMap<EntityId, usize>>,
    children_by_parent: Arc<HashMap<EntityId, Vec<EntityId>>>,
    child_positions: Arc<HashMap<EntityId, usize>>,
    child_hash_aggregates: Arc<[u64]>,
    child_hash_overrides: HierarchyChildHashOverrides,
    published_delta: Option<WorldInspectionDelta>,
    summary: WorldInspectionSummary,
}

/// Immutable hierarchy rows with sparse generation-local replacements.
///
/// World-sync consumers address changed rows through their delta anchors. A full row slice is
/// materialized only when a consumer explicitly asks for one, so a name edit does not allocate a
/// new full hierarchy merely to publish its small incremental change set.
#[derive(Debug)]
struct HierarchyRows {
    base: Arc<[WorldInspectionHierarchyRow]>,
    overrides: HierarchyRowOverrides,
    materialized: OnceLock<Arc<[WorldInspectionHierarchyRow]>>,
    materializations: Arc<HierarchyRowMaterializations>,
}

impl PartialEq for HierarchyRows {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl HierarchyRows {
    fn complete(
        rows: Arc<[WorldInspectionHierarchyRow]>,
        materializations: Arc<HierarchyRowMaterializations>,
    ) -> Self {
        Self {
            base: rows,
            overrides: HierarchyRowOverrides::default(),
            materialized: OnceLock::new(),
            materializations,
        }
    }

    fn with_overrides(previous: &Self, overrides: HierarchyRowOverrides) -> Self {
        Self {
            base: previous.base.clone(),
            overrides,
            materialized: OnceLock::new(),
            materializations: previous.materializations.clone(),
        }
    }

    fn clone_for_cache(&self) -> (Self, (u64, u64)) {
        let (materialized_rows, totals) = self
            .materializations
            .with_snapshot(|totals| (self.materialized.get().cloned(), totals));
        let materialized = OnceLock::new();
        if let Some(rows) = materialized_rows {
            let _ = materialized.set(rows.clone());
        }
        (
            Self {
                base: self.base.clone(),
                overrides: self.overrides.clone(),
                materialized,
                materializations: Arc::new(HierarchyRowMaterializations::from_totals(totals)),
            },
            totals,
        )
    }

    fn row(&self, index: usize) -> Option<&WorldInspectionHierarchyRow> {
        self.overrides.get(&index).or_else(|| self.base.get(index))
    }

    fn as_slice(&self) -> &[WorldInspectionHierarchyRow] {
        if self.overrides.is_empty() {
            return &self.base;
        }
        self.materialized_rows().as_ref()
    }

    fn as_arc(&self) -> Arc<[WorldInspectionHierarchyRow]> {
        if self.overrides.is_empty() {
            return self.base.clone();
        }
        self.materialized_rows().clone()
    }

    fn materialized_rows(&self) -> &Arc<[WorldInspectionHierarchyRow]> {
        self.materialized.get_or_init(|| {
            crate::profile_scope!("runtime", "scene_inspection", "hierarchy_rows_materialize");
            let mut rows = self.base.to_vec();
            self.overrides.for_each(|index, row| {
                if let Some(slot) = rows.get_mut(index) {
                    *slot = row.clone();
                }
            });
            let rows: Arc<[WorldInspectionHierarchyRow]> = rows.into();
            self.materializations.record(rows.len());
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            {
                let (full_materializations, complete_view_rows) = self.materializations.snapshot();
                crate::profile_counter!(
                    "runtime",
                    "scene_inspection_hierarchy_full_materializations",
                    full_materializations
                );
                crate::profile_counter!(
                    "runtime",
                    "scene_inspection_hierarchy_complete_view_rows_materialized",
                    complete_view_rows
                );
            }
            rows
        })
    }

    #[cfg(test)]
    fn override_count(&self) -> usize {
        self.overrides.len()
    }

    #[cfg(test)]
    fn is_materialized(&self) -> bool {
        self.materialized.get().is_some()
    }
}

impl WorldInspectionArtifact {
    pub(super) fn clone_for_cache(&self) -> (Self, (u64, u64)) {
        let (hierarchy_rows, materialization_totals) = self.hierarchy_rows.clone_for_cache();
        (
            Self {
                generation: self.generation,
                hierarchy_rows: Arc::new(hierarchy_rows),
                row_indices: self.row_indices.clone(),
                children_by_parent: self.children_by_parent.clone(),
                child_positions: self.child_positions.clone(),
                child_hash_aggregates: self.child_hash_aggregates.clone(),
                child_hash_overrides: self.child_hash_overrides.clone(),
                published_delta: self.published_delta.clone(),
                summary: self.summary,
            },
            materialization_totals,
        )
    }

    pub(super) fn from_world(world: &World, previous: Option<&Self>) -> Self {
        let nodes = world.node_records();
        let materializations = match previous {
            Some(previous) => previous.hierarchy_rows.materializations.clone(),
            None => Arc::default(),
        };
        let hierarchy_rows = Arc::new(HierarchyRows::complete(
            build_hierarchy_rows_from_nodes(world, &nodes).into(),
            materializations,
        ));
        let row_indices = Arc::new(
            hierarchy_rows
                .as_slice()
                .iter()
                .enumerate()
                .map(|(index, row)| (row.entity, index))
                .collect(),
        );
        let children_by_parent = Arc::new(hierarchy_children_by_parent(hierarchy_rows.as_slice()));
        let child_positions = Arc::new(hierarchy_child_positions(&children_by_parent));
        let child_hash_aggregates: Arc<[u64]> = hierarchy_child_hash_aggregates(
            hierarchy_rows.as_slice(),
            &row_indices,
            &children_by_parent,
        )
        .into();
        let generation = world.world_generation();
        let published_delta = previous.map(|previous| {
            hierarchy_delta_between(
                previous.generation,
                previous.hierarchy_rows.as_slice(),
                &previous.row_indices,
                generation,
                hierarchy_rows.as_slice(),
                &row_indices,
            )
        });
        Self {
            generation,
            summary: WorldInspectionSummary::from_nodes(&nodes, hierarchy_rows.as_slice()),
            hierarchy_rows,
            row_indices,
            children_by_parent,
            child_positions,
            child_hash_aggregates,
            child_hash_overrides: HierarchyChildHashOverrides::default(),
            published_delta,
        }
    }

    pub(super) fn from_previous_generation(previous: &Self, generation: u64) -> Self {
        Self {
            generation,
            hierarchy_rows: previous.hierarchy_rows.clone(),
            row_indices: previous.row_indices.clone(),
            children_by_parent: previous.children_by_parent.clone(),
            child_positions: previous.child_positions.clone(),
            child_hash_aggregates: previous.child_hash_aggregates.clone(),
            child_hash_overrides: previous.child_hash_overrides.clone(),
            published_delta: Some(WorldInspectionDelta {
                previous_generation: previous.generation,
                generation,
                hierarchy_reflow_required: false,
                added_rows: Vec::new(),
                changed_rows: Vec::new(),
                removed_entities: Vec::new(),
            }),
            summary: previous.summary,
        }
    }

    /// Rebuilds only the renamed rows and their ancestry anchors when topology is unchanged.
    ///
    /// The artifact retains base rows plus sparse immutable replacements, so the producer work,
    /// published delta, and subtree-hash recomputation remain bounded by dirty anchors. A full
    /// row slice is materialized later only for a consumer that explicitly requests it.
    pub(super) fn from_name_changes(
        world: &World,
        previous: &Self,
        dirty_names: &BTreeSet<EntityId>,
    ) -> Option<(Self, usize, usize)> {
        crate::profile_scope!("runtime", "scene_inspection", "hierarchy_name_delta");
        let mut overrides = previous.hierarchy_rows.overrides.clone();
        let mut child_hash_overrides = previous.child_hash_overrides.clone();
        let mut child_hash_update_count = 0usize;
        let mut affected = BTreeSet::new();

        for entity in dirty_names {
            let index = *previous.row_indices.get(entity)?;
            let current = world.node_record(*entity)?;
            let previous_row = previous.hierarchy_rows.row(index)?;
            if current.parent != previous_row.parent {
                return None;
            }
            let mut renamed_row = previous_row.clone();
            renamed_row.display_name = current.name;
            overrides.insert(index, renamed_row);

            let mut ancestor = Some(*entity);
            let mut ancestry = BTreeSet::new();
            while let Some(current) = ancestor {
                if !ancestry.insert(current) {
                    return None;
                }
                let row_index = *previous.row_indices.get(&current)?;
                affected.insert(current);
                ancestor = previous.hierarchy_rows.row(row_index)?.parent;
            }
        }

        let mut affected = affected.into_iter().collect::<Vec<_>>();
        affected.sort_by_key(|entity| {
            std::cmp::Reverse(
                previous
                    .row_indices
                    .get(entity)
                    .and_then(|index| previous.hierarchy_rows.row(*index))
                    .map_or(0, |row| row.depth),
            )
        });
        for entity in &affected {
            let row_index = *previous.row_indices.get(entity)?;
            let child_count = previous.children_by_parent.get(entity).map_or(0, Vec::len);
            let child_hash_aggregate = child_hash_overrides
                .get(&row_index)
                .copied()
                .or_else(|| previous.child_hash_aggregates.get(row_index).copied())?;
            let display_name = overrides
                .get(&row_index)
                .or_else(|| previous.hierarchy_rows.row(row_index))?
                .display_name
                .clone();
            let mut row = overrides
                .get(&row_index)
                .or_else(|| previous.hierarchy_rows.row(row_index))?
                .clone();
            let previous_subtree_hash = row.subtree_hash;
            row.subtree_hash = hierarchy_subtree_hash_from_child_aggregate(
                &display_name,
                child_count,
                child_hash_aggregate,
            );
            let subtree_hash = row.subtree_hash;
            let parent = row.parent;
            overrides.insert(row_index, row);

            if previous_subtree_hash != subtree_hash {
                if let Some(parent) = parent {
                    let parent_index = *previous.row_indices.get(&parent)?;
                    let parent_row = previous.hierarchy_rows.row(parent_index)?;
                    let current_row = previous.hierarchy_rows.row(row_index)?;
                    if current_row.depth != parent_row.depth.saturating_add(1) {
                        return None;
                    }
                    let ordinal = *previous.child_positions.get(entity)?;
                    let parent_aggregate = child_hash_overrides
                        .get(&parent_index)
                        .copied()
                        .or_else(|| previous.child_hash_aggregates.get(parent_index).copied())?
                        ^ hierarchy_child_hash_contribution(
                            ordinal,
                            *entity,
                            previous_subtree_hash,
                        )
                        ^ hierarchy_child_hash_contribution(ordinal, *entity, subtree_hash);
                    child_hash_overrides.insert(parent_index, parent_aggregate);
                    child_hash_update_count = child_hash_update_count.saturating_add(1);
                }
            }
        }

        let changed_rows = affected
            .iter()
            .filter_map(|entity| {
                let index = *previous.row_indices.get(entity)?;
                let row = overrides
                    .get(&index)
                    .or_else(|| previous.hierarchy_rows.row(index))?;
                (previous.hierarchy_rows.row(index)? != row).then(|| row.clone())
            })
            .collect::<Vec<_>>();
        let changed_row_count = changed_rows.len();
        let generation = world.world_generation();
        Some((
            Self {
                generation,
                hierarchy_rows: Arc::new(HierarchyRows::with_overrides(
                    previous.hierarchy_rows.as_ref(),
                    overrides,
                )),
                row_indices: previous.row_indices.clone(),
                children_by_parent: previous.children_by_parent.clone(),
                child_positions: previous.child_positions.clone(),
                child_hash_aggregates: previous.child_hash_aggregates.clone(),
                child_hash_overrides,
                published_delta: Some(WorldInspectionDelta {
                    previous_generation: previous.generation,
                    generation,
                    hierarchy_reflow_required: false,
                    added_rows: Vec::new(),
                    changed_rows,
                    removed_entities: Vec::new(),
                }),
                summary: previous.summary,
            },
            changed_row_count,
            child_hash_update_count,
        ))
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn hierarchy_rows(&self) -> &[WorldInspectionHierarchyRow] {
        self.hierarchy_rows.as_slice()
    }

    /// Clones the shared immutable hierarchy allocation for derived editor views.
    pub fn hierarchy_rows_arc(&self) -> Arc<[WorldInspectionHierarchyRow]> {
        self.hierarchy_rows.as_arc()
    }

    #[cfg(test)]
    pub(in crate::scene::inspection) fn hierarchy_row_override_count(&self) -> usize {
        self.hierarchy_rows.override_count()
    }

    #[cfg(test)]
    pub(in crate::scene::inspection) fn hierarchy_rows_are_materialized(&self) -> bool {
        self.hierarchy_rows.is_materialized()
    }

    #[cfg(test)]
    pub(in crate::scene::inspection) fn hierarchy_child_hash_override_count(&self) -> usize {
        self.child_hash_overrides.len()
    }

    /// Returns one hierarchy row from this immutable generation by stable entity identity.
    pub fn hierarchy_row(&self, entity: EntityId) -> Option<&WorldInspectionHierarchyRow> {
        self.row_indices
            .get(&entity)
            .and_then(|index| self.hierarchy_rows.row(*index))
    }

    pub(super) fn hierarchy_row_count(&self) -> usize {
        self.hierarchy_rows.as_slice().len()
    }

    pub(super) fn hierarchy_materialization_totals(&self) -> (u64, u64) {
        self.hierarchy_rows.materializations.snapshot()
    }

    pub const fn summary(&self) -> WorldInspectionSummary {
        self.summary
    }

    /// Returns the delta prepared while publishing the immediately preceding generation.
    pub fn published_delta_from(&self, previous_generation: u64) -> Option<&WorldInspectionDelta> {
        self.published_delta
            .as_ref()
            .filter(|delta| delta.previous_generation == previous_generation)
    }

    /// Computes a consumer delta only when a new immutable generation is published.
    pub fn delta_from(&self, previous: &Self) -> WorldInspectionDelta {
        hierarchy_delta_between(
            previous.generation,
            previous.hierarchy_rows.as_slice(),
            &previous.row_indices,
            self.generation,
            self.hierarchy_rows.as_slice(),
            &self.row_indices,
        )
    }
}

fn hierarchy_children_by_parent(
    hierarchy_rows: &[WorldInspectionHierarchyRow],
) -> HashMap<EntityId, Vec<EntityId>> {
    let mut children_by_parent = HashMap::<EntityId, Vec<EntityId>>::new();
    for row in hierarchy_rows {
        if let Some(parent) = row.parent {
            children_by_parent
                .entry(parent)
                .or_default()
                .push(row.entity);
        }
    }
    children_by_parent
}

fn hierarchy_child_positions(
    children_by_parent: &HashMap<EntityId, Vec<EntityId>>,
) -> HashMap<EntityId, usize> {
    children_by_parent
        .values()
        .flat_map(|children| children.iter().copied().enumerate())
        .map(|(ordinal, entity)| (entity, ordinal))
        .collect()
}

fn hierarchy_child_hash_aggregates(
    hierarchy_rows: &[WorldInspectionHierarchyRow],
    row_indices: &HashMap<EntityId, usize>,
    children_by_parent: &HashMap<EntityId, Vec<EntityId>>,
) -> Vec<u64> {
    let mut aggregates = vec![0; hierarchy_rows.len()];
    for (parent, children) in children_by_parent {
        let Some(parent_index) = row_indices.get(parent).copied() else {
            continue;
        };
        let Some(parent_row) = hierarchy_rows.get(parent_index) else {
            continue;
        };
        let aggregate =
            children
                .iter()
                .copied()
                .enumerate()
                .fold(0, |aggregate, (ordinal, child)| {
                    let child_hash = row_indices
                        .get(&child)
                        .and_then(|index| hierarchy_rows.get(*index))
                        .filter(|row| row.depth == parent_row.depth.saturating_add(1))
                        .map_or(0, |row| row.subtree_hash);
                    aggregate ^ hierarchy_child_hash_contribution(ordinal, child, child_hash)
                });
        aggregates[parent_index] = aggregate;
    }
    aggregates
}

fn hierarchy_delta_between(
    previous_generation: u64,
    previous_rows: &[WorldInspectionHierarchyRow],
    previous_row_indices: &HashMap<EntityId, usize>,
    generation: u64,
    hierarchy_rows: &[WorldInspectionHierarchyRow],
    row_indices: &HashMap<EntityId, usize>,
) -> WorldInspectionDelta {
    let added_rows = hierarchy_rows
        .iter()
        .filter(|row| !previous_row_indices.contains_key(&row.entity))
        .cloned()
        .collect();
    let changed_rows = hierarchy_rows
        .iter()
        .filter(|row| {
            previous_row_indices.get(&row.entity).is_some_and(|index| {
                previous_rows
                    .get(*index)
                    .is_some_and(|previous| previous != *row)
            })
        })
        .cloned()
        .collect();
    let removed_entities = previous_rows
        .iter()
        .filter(|row| !row_indices.contains_key(&row.entity))
        .map(|row| row.entity)
        .collect();

    WorldInspectionDelta {
        previous_generation,
        generation,
        hierarchy_reflow_required: true,
        added_rows,
        changed_rows,
        removed_entities,
    }
}

/// Aggregate scene facts that editor views can consume without another node scan.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldInspectionSummary {
    node_count: usize,
    visible_node_count: usize,
    camera_count: usize,
    mesh_count: usize,
    light_count: usize,
}

impl WorldInspectionSummary {
    fn from_nodes(nodes: &[SceneNode], hierarchy_rows: &[WorldInspectionHierarchyRow]) -> Self {
        let mut summary = Self {
            node_count: nodes.len(),
            visible_node_count: hierarchy_rows
                .iter()
                .filter(|row| row.active_in_hierarchy)
                .count(),
            ..Self::default()
        };
        for node in nodes {
            summary.camera_count += usize::from(node.camera.is_some());
            summary.mesh_count += usize::from(node.mesh.is_some());
            summary.light_count += usize::from(
                node.directional_light.is_some()
                    || node.ambient_light.is_some()
                    || node.point_light.is_some()
                    || node.rect_light.is_some()
                    || node.spot_light.is_some(),
            );
        }
        summary
    }

    pub const fn node_count(self) -> usize {
        self.node_count
    }

    pub const fn visible_node_count(self) -> usize {
        self.visible_node_count
    }

    pub const fn camera_count(self) -> usize {
        self.camera_count
    }

    pub const fn mesh_count(self) -> usize {
        self.mesh_count
    }

    pub const fn light_count(self) -> usize {
        self.light_count
    }
}

/// Entity-addressable hierarchy change set between two published artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldInspectionDelta {
    previous_generation: u64,
    generation: u64,
    hierarchy_reflow_required: bool,
    added_rows: Vec<WorldInspectionHierarchyRow>,
    changed_rows: Vec<WorldInspectionHierarchyRow>,
    removed_entities: Vec<EntityId>,
}

impl WorldInspectionDelta {
    pub const fn previous_generation(&self) -> u64 {
        self.previous_generation
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Whether the producer rebuilt the hierarchy and the consumer must reflow its view.
    ///
    /// Name-only refreshes preserve the previous order and topology, so they deliberately keep
    /// this false and may be applied as sparse patches.
    pub const fn requires_hierarchy_reflow(&self) -> bool {
        self.hierarchy_reflow_required
    }

    pub fn added_rows(&self) -> &[WorldInspectionHierarchyRow] {
        &self.added_rows
    }

    pub fn changed_rows(&self) -> &[WorldInspectionHierarchyRow] {
        &self.changed_rows
    }

    pub fn removed_entities(&self) -> &[EntityId] {
        &self.removed_entities
    }
}
