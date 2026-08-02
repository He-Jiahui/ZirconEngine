use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, RwLock};

use crate::scene::components::SceneNode;
use crate::scene::{EntityId, World};

use super::snapshot::{build_hierarchy_rows_from_nodes, build_inspection_fields};
use super::{WorldInspectionField, WorldInspectionHierarchyRow};

/// Immutable, generation-scoped runtime data shared by scene inspection consumers.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldInspectionArtifact {
    generation: u64,
    hierarchy_rows: Arc<[WorldInspectionHierarchyRow]>,
    row_indices: Arc<HashMap<EntityId, usize>>,
    published_delta: Option<WorldInspectionDelta>,
    summary: WorldInspectionSummary,
}

impl WorldInspectionArtifact {
    fn from_world(world: &World, previous: Option<&Self>) -> Self {
        let nodes = world.node_records();
        let hierarchy_rows: Arc<[WorldInspectionHierarchyRow]> =
            build_hierarchy_rows_from_nodes(world, &nodes, None).into();
        let row_indices = Arc::new(
            hierarchy_rows
                .iter()
                .enumerate()
                .map(|(index, row)| (row.entity, index))
                .collect(),
        );
        let generation = world.world_generation();
        let published_delta = previous.map(|previous| {
            hierarchy_delta_between(
                previous.generation,
                &previous.hierarchy_rows,
                &previous.row_indices,
                generation,
                &hierarchy_rows,
                &row_indices,
            )
        });
        Self {
            generation,
            summary: WorldInspectionSummary::from_nodes(&nodes, &hierarchy_rows),
            hierarchy_rows,
            row_indices,
            published_delta,
        }
    }

    fn from_previous_generation(previous: &Self, generation: u64) -> Self {
        Self {
            generation,
            hierarchy_rows: previous.hierarchy_rows.clone(),
            row_indices: previous.row_indices.clone(),
            published_delta: Some(WorldInspectionDelta {
                previous_generation: previous.generation,
                generation,
                added_rows: Vec::new(),
                changed_rows: Vec::new(),
                removed_entities: Vec::new(),
            }),
            summary: previous.summary,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn hierarchy_rows(&self) -> &[WorldInspectionHierarchyRow] {
        &self.hierarchy_rows
    }

    /// Clones the shared immutable hierarchy allocation for derived editor views.
    pub fn hierarchy_rows_arc(&self) -> Arc<[WorldInspectionHierarchyRow]> {
        self.hierarchy_rows.clone()
    }

    /// Returns one hierarchy row from this immutable generation by stable entity identity.
    pub fn hierarchy_row(&self, entity: EntityId) -> Option<&WorldInspectionHierarchyRow> {
        self.row_indices
            .get(&entity)
            .and_then(|index| self.hierarchy_rows.get(*index))
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
            &previous.hierarchy_rows,
            &previous.row_indices,
            self.generation,
            &self.hierarchy_rows,
            &self.row_indices,
        )
    }
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
        added_rows,
        changed_rows,
        removed_entities,
    }
}

/// Immutable inspector payload for one entity in one runtime generation.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldInspectionFieldsArtifact {
    generation: u64,
    entity: EntityId,
    fields: Arc<[WorldInspectionField]>,
}

impl WorldInspectionFieldsArtifact {
    fn from_world(world: &World, entity: EntityId) -> Self {
        Self {
            generation: world.world_generation(),
            entity,
            fields: build_inspection_fields(world, entity).into(),
        }
    }

    fn from_previous_generation(previous: &Self, generation: u64) -> Self {
        Self {
            generation,
            entity: previous.entity,
            fields: previous.fields.clone(),
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub fn fields(&self) -> &[WorldInspectionField] {
        &self.fields
    }

    /// Computes changes for a single entity's inspector fields across generations.
    pub fn delta_from(&self, previous: &Self) -> WorldInspectionFieldDelta {
        let previous_fields = previous
            .fields
            .iter()
            .map(|field| {
                (
                    (
                        field.component_type_path.as_str(),
                        field.field_name.as_str(),
                    ),
                    field,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let current_fields = self
            .fields
            .iter()
            .map(|field| {
                (
                    (
                        field.component_type_path.as_str(),
                        field.field_name.as_str(),
                    ),
                    field,
                )
            })
            .collect::<BTreeMap<_, _>>();

        let changed_fields = self
            .fields
            .iter()
            .filter(|field| {
                previous_fields
                    .get(&(
                        field.component_type_path.as_str(),
                        field.field_name.as_str(),
                    ))
                    .is_none_or(|previous_field| *previous_field != *field)
            })
            .cloned()
            .collect();
        let removed_fields = previous
            .fields
            .iter()
            .filter(|field| {
                !current_fields.contains_key(&(
                    field.component_type_path.as_str(),
                    field.field_name.as_str(),
                ))
            })
            .map(WorldInspectionFieldPath::from_field)
            .collect();

        WorldInspectionFieldDelta {
            previous_generation: previous.generation,
            generation: self.generation,
            entity: self.entity,
            changed_fields,
            removed_fields,
        }
    }
}

/// Stable identity for an inspector property in a runtime artifact.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldInspectionFieldPath {
    component_type_path: String,
    field_name: String,
}

impl WorldInspectionFieldPath {
    fn from_field(field: &WorldInspectionField) -> Self {
        Self {
            component_type_path: field.component_type_path.clone(),
            field_name: field.field_name.clone(),
        }
    }

    pub fn component_type_path(&self) -> &str {
        &self.component_type_path
    }

    pub fn field_name(&self) -> &str {
        &self.field_name
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

/// Entity/property-addressable inspector changes between focused-field artifacts.
#[derive(Clone, Debug, PartialEq)]
pub struct WorldInspectionFieldDelta {
    previous_generation: u64,
    generation: u64,
    entity: EntityId,
    changed_fields: Vec<WorldInspectionField>,
    removed_fields: Vec<WorldInspectionFieldPath>,
}

impl WorldInspectionFieldDelta {
    pub const fn previous_generation(&self) -> u64 {
        self.previous_generation
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub fn changed_fields(&self) -> &[WorldInspectionField] {
        &self.changed_fields
    }

    pub fn removed_fields(&self) -> &[WorldInspectionFieldPath] {
        &self.removed_fields
    }
}

/// Cumulative producer work recorded by the runtime inspection artifact cache.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorldInspectionArtifactDiagnostics {
    hierarchy_builds: u64,
    hierarchy_rows_built: u64,
    focused_field_builds: u64,
    focused_fields_built: u64,
}

impl WorldInspectionArtifactDiagnostics {
    pub const fn hierarchy_builds(self) -> u64 {
        self.hierarchy_builds
    }

    pub const fn hierarchy_rows_built(self) -> u64 {
        self.hierarchy_rows_built
    }

    pub const fn focused_field_builds(self) -> u64 {
        self.focused_field_builds
    }

    pub const fn focused_fields_built(self) -> u64 {
        self.focused_fields_built
    }
}

#[derive(Debug)]
pub(in crate::scene) struct WorldInspectionArtifactCache {
    artifact: RwLock<Option<Arc<WorldInspectionArtifact>>>,
    // The editor publishes one primary selection at a time and retains the
    // prior Arc itself while it computes a delta. Keeping only the current
    // focused artifact prevents generation publication from scaling with every
    // entity selected over the lifetime of a large scene.
    fields: RwLock<Option<Arc<WorldInspectionFieldsArtifact>>>,
    dirty_field_entities: RwLock<BTreeSet<EntityId>>,
    diagnostics: RwLock<WorldInspectionArtifactDiagnostics>,
    hierarchy_rows_dirty: RwLock<bool>,
}

impl Default for WorldInspectionArtifactCache {
    fn default() -> Self {
        Self {
            artifact: RwLock::new(None),
            fields: RwLock::new(None),
            dirty_field_entities: RwLock::new(BTreeSet::new()),
            diagnostics: RwLock::new(WorldInspectionArtifactDiagnostics::default()),
            hierarchy_rows_dirty: RwLock::new(true),
        }
    }
}

impl Clone for WorldInspectionArtifactCache {
    fn clone(&self) -> Self {
        Self {
            artifact: RwLock::new(self.current_any_generation()),
            fields: RwLock::new(self.current_fields()),
            dirty_field_entities: RwLock::new(self.dirty_field_entities()),
            diagnostics: RwLock::new(self.diagnostics()),
            hierarchy_rows_dirty: RwLock::new(self.hierarchy_rows_dirty()),
        }
    }
}

impl PartialEq for WorldInspectionArtifactCache {
    fn eq(&self, _other: &Self) -> bool {
        // The cache is a runtime-only derived view, not persistent world state.
        true
    }
}

impl WorldInspectionArtifactCache {
    fn current(&self, generation: u64) -> Option<Arc<WorldInspectionArtifact>> {
        self.artifact
            .read()
            .expect("world inspection artifact cache lock poisoned")
            .as_ref()
            .filter(|artifact| artifact.generation == generation)
            .cloned()
    }

    fn current_any_generation(&self) -> Option<Arc<WorldInspectionArtifact>> {
        self.artifact
            .read()
            .expect("world inspection artifact cache lock poisoned")
            .clone()
    }

    fn store(&self, artifact: Arc<WorldInspectionArtifact>) {
        let generation = artifact.generation();
        *self
            .artifact
            .write()
            .expect("world inspection artifact cache lock poisoned") = Some(artifact.clone());
        let dirty_field_entities = std::mem::take(
            &mut *self
                .dirty_field_entities
                .write()
                .expect("world inspection field dirty set lock poisoned"),
        );
        let mut fields = self
            .fields
            .write()
            .expect("world inspection field artifact cache lock poisoned");
        let can_reuse_cached_fields = fields.as_ref().is_some_and(|cached| {
            artifact.hierarchy_row(cached.entity()).is_some()
                && !dirty_field_entities.contains(&cached.entity())
        });
        if can_reuse_cached_fields {
            let cached = fields
                .as_mut()
                .expect("cached inspection fields must remain present");
            *cached = Arc::new(WorldInspectionFieldsArtifact::from_previous_generation(
                cached, generation,
            ));
        } else {
            *fields = None;
        }
    }

    pub(in crate::scene) fn mark_hierarchy_rows_dirty(&self) {
        *self
            .hierarchy_rows_dirty
            .write()
            .expect("world inspection hierarchy cache lock poisoned") = true;
    }

    pub(in crate::scene) fn mark_fields_dirty(&self, entity: EntityId) {
        self.dirty_field_entities
            .write()
            .expect("world inspection field dirty set lock poisoned")
            .insert(entity);
    }

    fn mark_hierarchy_rows_clean(&self) {
        *self
            .hierarchy_rows_dirty
            .write()
            .expect("world inspection hierarchy cache lock poisoned") = false;
    }

    fn hierarchy_rows_dirty(&self) -> bool {
        *self
            .hierarchy_rows_dirty
            .read()
            .expect("world inspection hierarchy cache lock poisoned")
    }

    fn fields(
        &self,
        generation: u64,
        entity: EntityId,
    ) -> Option<Arc<WorldInspectionFieldsArtifact>> {
        self.fields
            .read()
            .expect("world inspection field artifact cache lock poisoned")
            .as_ref()
            .filter(|artifact| artifact.generation == generation && artifact.entity == entity)
            .cloned()
    }

    fn current_fields(&self) -> Option<Arc<WorldInspectionFieldsArtifact>> {
        self.fields
            .read()
            .expect("world inspection field artifact cache lock poisoned")
            .clone()
    }

    fn dirty_field_entities(&self) -> BTreeSet<EntityId> {
        self.dirty_field_entities
            .read()
            .expect("world inspection field dirty set lock poisoned")
            .clone()
    }

    fn store_fields(&self, artifact: Arc<WorldInspectionFieldsArtifact>) {
        *self
            .fields
            .write()
            .expect("world inspection field artifact cache lock poisoned") = Some(artifact);
    }

    fn record_hierarchy_build(&self, row_count: usize) {
        let mut diagnostics = self
            .diagnostics
            .write()
            .expect("world inspection artifact diagnostics lock poisoned");
        diagnostics.hierarchy_builds = diagnostics.hierarchy_builds.saturating_add(1);
        diagnostics.hierarchy_rows_built = diagnostics
            .hierarchy_rows_built
            .saturating_add(row_count as u64);
    }

    fn record_focused_field_build(&self, field_count: usize) {
        let mut diagnostics = self
            .diagnostics
            .write()
            .expect("world inspection artifact diagnostics lock poisoned");
        diagnostics.focused_field_builds = diagnostics.focused_field_builds.saturating_add(1);
        diagnostics.focused_fields_built = diagnostics
            .focused_fields_built
            .saturating_add(field_count as u64);
    }

    fn diagnostics(&self) -> WorldInspectionArtifactDiagnostics {
        *self
            .diagnostics
            .read()
            .expect("world inspection artifact diagnostics lock poisoned")
    }
}

impl World {
    /// Returns the immutable hierarchy artifact for the current runtime generation.
    /// Repeated reads at a stable generation reuse the same allocation.
    pub fn inspection_artifact(&self) -> Arc<WorldInspectionArtifact> {
        let generation = self.world_generation();
        if let Some(artifact) = self.inspection_artifact_cache.current(generation) {
            return artifact;
        }

        let previous = self
            .inspection_artifact_cache
            .current_any_generation()
            .filter(|artifact| artifact.generation != generation);
        let hierarchy_rows_dirty = self.inspection_artifact_cache.hierarchy_rows_dirty();
        let rebuilt_hierarchy = previous.is_none() || hierarchy_rows_dirty;
        let artifact = Arc::new(match previous.as_deref() {
            Some(previous) if !hierarchy_rows_dirty => {
                WorldInspectionArtifact::from_previous_generation(previous, generation)
            }
            _ => WorldInspectionArtifact::from_world(self, previous.as_deref()),
        });
        if rebuilt_hierarchy {
            self.inspection_artifact_cache
                .record_hierarchy_build(artifact.hierarchy_rows.len());
        }
        self.inspection_artifact_cache.store(artifact.clone());
        self.inspection_artifact_cache.mark_hierarchy_rows_clean();
        artifact
    }

    /// Returns cumulative work performed to publish immutable inspection artifacts.
    pub fn inspection_artifact_diagnostics(&self) -> WorldInspectionArtifactDiagnostics {
        self.inspection_artifact_cache.diagnostics()
    }

    /// Returns reflected Inspector fields for the current primary selection.
    /// Repeated reads for that selection reuse the immutable artifact until the world changes
    /// or another entity becomes primary; consumers retain prior artifacts for delta comparison.
    pub fn inspection_fields_artifact(
        &self,
        entity: EntityId,
    ) -> Option<Arc<WorldInspectionFieldsArtifact>> {
        if !self.contains_entity(entity) {
            return None;
        }

        let generation = self.inspection_artifact().generation();
        if let Some(artifact) = self.inspection_artifact_cache.fields(generation, entity) {
            return Some(artifact);
        }

        let artifact = Arc::new(WorldInspectionFieldsArtifact::from_world(self, entity));
        self.inspection_artifact_cache
            .record_focused_field_build(artifact.fields.len());
        self.inspection_artifact_cache
            .store_fields(artifact.clone());
        Some(artifact)
    }
}
