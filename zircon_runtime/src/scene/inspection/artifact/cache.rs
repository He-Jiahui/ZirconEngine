use std::collections::BTreeSet;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::scene::{EntityId, World};

use super::data::WorldInspectionArtifact;
use super::fields::WorldInspectionFieldsArtifact;
use super::metrics::WorldInspectionArtifactDiagnostics;

#[derive(Debug)]
pub(in crate::scene) struct WorldInspectionArtifactCache {
    artifact: RwLock<Option<Arc<WorldInspectionArtifact>>>,
    // The publisher retains the previous focused artifact while computing its delta. Keeping only
    // the current fields here prevents cache growth from following lifetime selection history.
    fields: RwLock<Option<Arc<WorldInspectionFieldsArtifact>>>,
    dirty_field_entities: RwLock<BTreeSet<EntityId>>,
    diagnostics: RwLock<WorldInspectionArtifactDiagnostics>,
    hierarchy_full_rebuild_required: RwLock<bool>,
    dirty_hierarchy_names: RwLock<BTreeSet<EntityId>>,
}

impl Default for WorldInspectionArtifactCache {
    fn default() -> Self {
        Self {
            artifact: RwLock::new(None),
            fields: RwLock::new(None),
            dirty_field_entities: RwLock::new(BTreeSet::new()),
            diagnostics: RwLock::new(WorldInspectionArtifactDiagnostics::default()),
            hierarchy_full_rebuild_required: RwLock::new(true),
            dirty_hierarchy_names: RwLock::new(BTreeSet::new()),
        }
    }
}

impl WorldInspectionArtifactCache {
    pub(in crate::scene) fn clone_for_world_generation(&self, world_generation: u64) -> Self {
        let mut diagnostics = *read_derived_cache(&self.diagnostics);
        let artifact = self.current_any_generation().map(|artifact| {
            let (artifact, materialization_totals) = artifact.clone_for_cache();
            (
                diagnostics.hierarchy_full_materializations,
                diagnostics.hierarchy_rows_materialized,
            ) = materialization_totals;
            Arc::new(artifact)
        });
        let hierarchy_full_rebuild_required = self.hierarchy_full_rebuild_required()
            || artifact
                .as_ref()
                .is_some_and(|artifact| artifact.generation() != world_generation);
        Self {
            artifact: RwLock::new(artifact),
            fields: RwLock::new(self.current_fields()),
            dirty_field_entities: RwLock::new(self.dirty_field_entities()),
            diagnostics: RwLock::new(diagnostics),
            hierarchy_full_rebuild_required: RwLock::new(hierarchy_full_rebuild_required),
            dirty_hierarchy_names: RwLock::new(self.dirty_hierarchy_names()),
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
        read_derived_cache(&self.artifact)
            .as_ref()
            .filter(|artifact| artifact.generation() == generation)
            .cloned()
    }

    fn current_any_generation(&self) -> Option<Arc<WorldInspectionArtifact>> {
        read_derived_cache(&self.artifact).clone()
    }

    fn store(&self, artifact: Arc<WorldInspectionArtifact>) {
        let generation = artifact.generation();
        *write_derived_cache(&self.artifact) = Some(artifact.clone());
        let dirty_field_entities =
            std::mem::take(&mut *write_derived_cache(&self.dirty_field_entities));
        let mut fields = write_derived_cache(&self.fields);
        let next_fields = fields
            .as_ref()
            .filter(|cached| {
                artifact.hierarchy_row(cached.entity()).is_some()
                    && !dirty_field_entities.contains(&cached.entity())
            })
            .map(|cached| {
                Arc::new(WorldInspectionFieldsArtifact::from_previous_generation(
                    cached, generation,
                ))
            });
        *fields = next_fields;
    }

    pub(in crate::scene) fn mark_hierarchy_rows_dirty(&self) {
        *write_derived_cache(&self.hierarchy_full_rebuild_required) = true;
        write_derived_cache(&self.dirty_hierarchy_names).clear();
    }

    /// Records a name-only edit that can update existing hierarchy anchors in place.
    pub(in crate::scene) fn mark_hierarchy_name_dirty(&self, entity: EntityId) {
        if self.hierarchy_full_rebuild_required() {
            return;
        }
        write_derived_cache(&self.dirty_hierarchy_names).insert(entity);
    }

    pub(in crate::scene) fn mark_fields_dirty(&self, entity: EntityId) {
        write_derived_cache(&self.dirty_field_entities).insert(entity);
    }

    fn mark_hierarchy_rows_clean(&self) {
        *write_derived_cache(&self.hierarchy_full_rebuild_required) = false;
        write_derived_cache(&self.dirty_hierarchy_names).clear();
    }

    fn hierarchy_full_rebuild_required(&self) -> bool {
        *read_derived_cache(&self.hierarchy_full_rebuild_required)
    }

    fn dirty_hierarchy_names(&self) -> BTreeSet<EntityId> {
        read_derived_cache(&self.dirty_hierarchy_names).clone()
    }

    fn fields(
        &self,
        generation: u64,
        entity: EntityId,
    ) -> Option<Arc<WorldInspectionFieldsArtifact>> {
        read_derived_cache(&self.fields)
            .as_ref()
            .filter(|artifact| artifact.generation() == generation && artifact.entity() == entity)
            .cloned()
    }

    fn current_fields(&self) -> Option<Arc<WorldInspectionFieldsArtifact>> {
        read_derived_cache(&self.fields).clone()
    }

    fn dirty_field_entities(&self) -> BTreeSet<EntityId> {
        read_derived_cache(&self.dirty_field_entities).clone()
    }

    fn store_fields(&self, artifact: Arc<WorldInspectionFieldsArtifact>) {
        *write_derived_cache(&self.fields) = Some(artifact);
    }

    fn record_hierarchy_build(&self, row_count: usize, child_hash_update_count: usize) {
        let _hierarchy_child_hash_updates = {
            let mut diagnostics = write_derived_cache(&self.diagnostics);
            diagnostics.hierarchy_builds = diagnostics.hierarchy_builds.saturating_add(1);
            diagnostics.hierarchy_rows_built = diagnostics
                .hierarchy_rows_built
                .saturating_add(row_count as u64);
            diagnostics.hierarchy_child_hash_updates = diagnostics
                .hierarchy_child_hash_updates
                .saturating_add(child_hash_update_count as u64);
            diagnostics.hierarchy_child_hash_updates
        };
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        crate::profile_counter!(
            "runtime",
            "scene_inspection_hierarchy_child_hash_updates",
            _hierarchy_child_hash_updates
        );
    }

    fn record_focused_field_build(&self, field_count: usize) {
        let mut diagnostics = write_derived_cache(&self.diagnostics);
        diagnostics.focused_field_builds = diagnostics.focused_field_builds.saturating_add(1);
        diagnostics.focused_fields_built = diagnostics
            .focused_fields_built
            .saturating_add(field_count as u64);
    }

    fn diagnostics(&self) -> WorldInspectionArtifactDiagnostics {
        let mut diagnostics = *read_derived_cache(&self.diagnostics);
        if let Some(artifact) = self.current_any_generation() {
            (
                diagnostics.hierarchy_full_materializations,
                diagnostics.hierarchy_rows_materialized,
            ) = artifact.hierarchy_materialization_totals();
        }
        diagnostics
    }
}

impl World {
    /// Returns the immutable hierarchy artifact for the current runtime generation.
    /// Repeated reads at a stable generation reuse the same allocation.
    pub fn inspection_artifact(&self) -> Arc<WorldInspectionArtifact> {
        crate::profile_scope!("runtime", "scene_inspection", "artifact_publish");
        let generation = self.world_generation();
        if let Some(artifact) = self.inspection_artifact_cache.current(generation) {
            return artifact;
        }

        let previous = self
            .inspection_artifact_cache
            .current_any_generation()
            .filter(|artifact| artifact.generation() != generation);
        let hierarchy_full_rebuild_required = self
            .inspection_artifact_cache
            .hierarchy_full_rebuild_required();
        let dirty_hierarchy_names = self.inspection_artifact_cache.dirty_hierarchy_names();
        let (artifact, rebuilt_row_count, child_hash_update_count) = match previous.as_deref() {
            Some(previous)
                if !hierarchy_full_rebuild_required && dirty_hierarchy_names.is_empty() =>
            {
                (
                    WorldInspectionArtifact::from_previous_generation(previous, generation),
                    0,
                    0,
                )
            }
            Some(previous) if !hierarchy_full_rebuild_required => {
                match WorldInspectionArtifact::from_name_changes(
                    self,
                    previous,
                    &dirty_hierarchy_names,
                ) {
                    Some(artifact) => artifact,
                    None => rebuild_artifact(self, Some(previous)),
                }
            }
            _ => rebuild_artifact(self, previous.as_deref()),
        };
        let artifact = Arc::new(artifact);
        if rebuilt_row_count != 0 {
            self.inspection_artifact_cache
                .record_hierarchy_build(rebuilt_row_count, child_hash_update_count);
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
            .record_focused_field_build(artifact.fields().len());
        self.inspection_artifact_cache
            .store_fields(artifact.clone());
        Some(artifact)
    }
}

fn rebuild_artifact(
    world: &World,
    previous: Option<&WorldInspectionArtifact>,
) -> (WorldInspectionArtifact, usize, usize) {
    let artifact = WorldInspectionArtifact::from_world(world, previous);
    let row_count = artifact.hierarchy_row_count();
    (artifact, row_count, 0)
}

fn read_derived_cache<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn write_derived_cache<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::RwLock;

    use crate::scene::NodeKind;

    use super::{WorldInspectionArtifactCache, WorldInspectionArtifactDiagnostics};
    use crate::scene::World;

    #[test]
    fn clone_generation_guard_rebuilds_a_split_publication_snapshot() {
        let mut source = World::empty();
        let renamed = source.spawn_node(NodeKind::Empty);
        let stale_artifact = source.inspection_artifact();
        source.rename_node(renamed, "Current name").unwrap();
        let expected_generation = source.world_generation();
        source.inspection_artifact();

        let split_publication_cache = WorldInspectionArtifactCache {
            artifact: RwLock::new(Some(stale_artifact)),
            fields: RwLock::new(None),
            dirty_field_entities: RwLock::new(BTreeSet::new()),
            diagnostics: RwLock::new(WorldInspectionArtifactDiagnostics::default()),
            hierarchy_full_rebuild_required: RwLock::new(false),
            dirty_hierarchy_names: RwLock::new(BTreeSet::new()),
        };
        let mut cloned = source.clone();
        cloned.inspection_artifact_cache =
            split_publication_cache.clone_for_world_generation(expected_generation);

        let artifact = cloned.inspection_artifact();
        assert_eq!(artifact.generation(), expected_generation);
        assert_eq!(
            artifact
                .hierarchy_row(renamed)
                .map(|row| row.display_name.as_str()),
            Some("Current name")
        );
    }
}
