use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::scene::components::{CameraComponent, Hierarchy, NodeKind};
use crate::scene::ecs::{
    ArchetypeSignature, ComponentId, ComponentTicks, DetachedEntityBatchOperationStats,
    DetachedEntityObservers, LifecycleEventKind, StoredComponent, TransferredComponentRow,
};
use crate::scene::EntityId;

use super::super::{SceneError, SceneResult, World};

mod preparation_owner;
mod prepared;
mod restore_parent_validation;
mod root_normalization;

pub(in crate::scene::world) use preparation_owner::DetachPreparationOwner;
pub use prepared::PreparedEntitySubtrees;

#[cfg(test)]
#[path = "detached_entity_batch/tests/raw_cycle_tests.rs"]
mod raw_cycle_tests;
#[cfg(test)]
#[path = "detached_entity_batch/tests/restore_cycle_profile.rs"]
mod restore_cycle_profile;
#[cfg(test)]
#[path = "detached_entity_batch/tests/restore_cycle_tests.rs"]
mod restore_cycle_tests;
#[cfg(test)]
#[path = "detached_entity_batch/tests/root_normalization_profile.rs"]
mod root_normalization_profile;
#[cfg(test)]
#[path = "detached_entity_batch/tests/cases.rs"]
mod tests;

/// Move-only ownership of recursively detached entity rows. The payload keeps
/// exact erased values and change ticks rather than cloning a world snapshot.
pub struct DetachedEntityBatch {
    entries: Vec<DetachedEntityBatchEntry>,
    restore_order: Vec<usize>,
    detached_active_camera: Option<EntityId>,
}

impl DetachedEntityBatch {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entity_ids(&self) -> impl ExactSizeIterator<Item = EntityId> + '_ {
        self.entries.iter().map(|entry| entry.entity)
    }

    fn operation_stats(&self) -> DetachedEntityBatchOperationStats {
        self.entries.iter().fold(
            DetachedEntityBatchOperationStats::default(),
            |mut stats, entry| {
                stats.moved_rows = stats.moved_rows.saturating_add(1);
                stats.moved_table_components = stats
                    .moved_table_components
                    .saturating_add(entry.table_components.len() as u64);
                stats.moved_sparse_components = stats
                    .moved_sparse_components
                    .saturating_add(entry.sparse_components.len() as u64);
                stats.moved_dynamic_components = stats.moved_dynamic_components.saturating_add(
                    entry
                        .dynamic_components
                        .as_ref()
                        .map_or(0, |components| components.len() as u64),
                );
                stats.archetype_publications = stats.archetype_publications.saturating_add(1);
                stats.lifecycle_events = stats.lifecycle_events.saturating_add(
                    entry.signature.table_components().len() as u64 * 2
                        + entry.signature.sparse_set_components().len() as u64 * 2,
                );
                stats
            },
        )
    }
}

struct DetachedEntityBatchEntry {
    entity: EntityId,
    kind: NodeKind,
    stable_order: usize,
    signature: ArchetypeSignature,
    table_components: BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
    sparse_components: Vec<TransferredComponentRow>,
    dynamic_components: Option<HashMap<String, serde_json::Value>>,
    observers: DetachedEntityObservers,
}

/// Owns a rejected batch so callers can retry or retain it without rebuilding
/// the affected scene state.
pub struct DetachedEntityBatchRestoreError {
    error: SceneError,
    batch: DetachedEntityBatch,
}

impl std::fmt::Debug for DetachedEntityBatchRestoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DetachedEntityBatchRestoreError")
            .field("error", &self.error)
            .field("batch_entries", &self.batch.entries.len())
            .finish()
    }
}

impl std::fmt::Display for DetachedEntityBatchRestoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.error, formatter)
    }
}

impl std::error::Error for DetachedEntityBatchRestoreError {}

impl DetachedEntityBatchRestoreError {
    pub fn error(&self) -> &SceneError {
        &self.error
    }

    pub fn into_parts(self) -> (SceneError, DetachedEntityBatch) {
        (self.error, self.batch)
    }
}

impl World {
    /// Detaches a root and its descendants without cloning the World. All
    /// lookup and signature checks complete before the first archetype row is
    /// removed; the following row transfer is therefore infallible.
    pub fn remove_entity_recursive(&mut self, root: EntityId) -> SceneResult<DetachedEntityBatch> {
        self.remove_entity_subtrees([root])
    }

    /// Detaches the union of the requested subtrees. Duplicate roots and roots
    /// already covered by another requested ancestor are normalized before
    /// preflight, so every affected entity is moved exactly once.
    pub fn remove_entity_subtrees(
        &mut self,
        roots: impl IntoIterator<Item = EntityId>,
    ) -> SceneResult<DetachedEntityBatch> {
        let prepared = self.prepare_entity_subtrees(roots)?;
        self.remove_prepared_entity_subtrees(prepared)
    }

    /// Preflights indexed subtree rows without removing them or publishing
    /// lifecycle events. The returned counts remain valid only in this World
    /// at the captured generation; stale preparations must be rebuilt.
    /// 预备前会先归并已有 Query 可变访问的通知；拒绝预检也会记录诊断，因此不保证所有运行态字段零变化。
    pub fn prepare_entity_subtrees(
        &mut self,
        roots: impl IntoIterator<Item = EntityId>,
    ) -> SceneResult<PreparedEntitySubtrees> {
        self.flush_deferred_component_mutations();
        let world_generation = self.world_generation();
        if world_generation == u64::MAX {
            self.record_detached_entity_batch_rejected_preflight();
            return Err(SceneError::DetachedEntityPreparationGenerationExhausted);
        }
        let roots = roots.into_iter().collect::<BTreeSet<_>>();
        if roots.is_empty() {
            self.record_detached_entity_batch_rejected_preflight();
            return Err(SceneError::DetachedEntityBatchInvariant {
                reason: "detached entity root set is empty",
            });
        }
        for root in roots.iter().copied() {
            if !self.contains_entity(root) {
                self.record_detached_entity_batch_rejected_preflight();
                return Err(SceneError::missing_entity("detach", root));
            }
        }

        let mut normalized_roots =
            match root_normalization::normalize_roots(&roots, |entity| self.parent_of(entity)) {
                Ok(roots) => roots,
                Err(error) => {
                    self.record_detached_entity_batch_rejected_preflight();
                    return Err(error);
                }
            };
        let hierarchy_index_rebuild_rows = self.ensure_hierarchy_mutation_index_current();
        normalized_roots.sort_unstable_by_key(|root| {
            self.stable_entity_order(*root)
                .expect("validated detached root must retain stable order")
        });
        let mut entities_by_order = BTreeMap::new();
        let mut detach_preorder = Vec::new();
        for root in normalized_roots.iter().copied() {
            for entity in self.subtree_entity_ids(root) {
                let order = self
                    .stable_entity_order(entity)
                    .expect("indexed detached entity must retain stable order");
                let previous = entities_by_order.insert(order, entity);
                debug_assert!(previous.is_none() || previous == Some(entity));
                if previous.is_none() {
                    detach_preorder.push(entity);
                }
            }
        }
        let entities = entities_by_order.into_values().collect::<Vec<_>>();
        let stable_batch_indices = entities
            .iter()
            .copied()
            .enumerate()
            .map(|(index, entity)| (entity, index))
            .collect::<HashMap<_, _>>();
        let restore_order = detach_preorder
            .iter()
            .map(|entity| {
                *stable_batch_indices
                    .get(entity)
                    .expect("detached preorder entity must exist in stable batch order")
            })
            .collect::<Vec<_>>();
        if let Err(error) = self.preflight_detached_entities(&entities) {
            self.record_detached_entity_batch_rejected_preflight();
            return Err(error);
        }

        let affected_camera_count =
            self.registered_component_id::<CameraComponent>()
                .map_or(0, |component_id| {
                    entities
                        .iter()
                        .filter(|entity| self.contains_component_id(**entity, component_id))
                        .count()
                });
        Ok(PreparedEntitySubtrees {
            owner: self.detach_preparation_owner.clone(),
            world_generation,
            normalized_roots,
            entities,
            detach_preorder,
            restore_order,
            world_camera_count: self.camera_count(),
            affected_camera_count,
            hierarchy_index_rebuild_rows,
        })
    }

    /// Commits exactly the preflighted subtree union. A foreign or stale
    /// preparation is rejected before any rows or lifecycle state are changed.
    pub fn remove_prepared_entity_subtrees(
        &mut self,
        prepared: PreparedEntitySubtrees,
    ) -> SceneResult<DetachedEntityBatch> {
        if !self.detach_preparation_owner.is_same_world(&prepared.owner) {
            self.record_detached_entity_batch_rejected_preflight();
            return Err(SceneError::DetachedEntityPreparationWorldMismatch);
        }
        let current_generation = self.world_generation();
        if prepared.world_generation != current_generation {
            self.record_detached_entity_batch_rejected_preflight();
            return Err(SceneError::DetachedEntityPreparationStale {
                prepared_generation: prepared.world_generation,
                current_generation,
            });
        }
        let PreparedEntitySubtrees {
            entities,
            detach_preorder,
            restore_order,
            hierarchy_index_rebuild_rows,
            ..
        } = prepared;
        let detached_active_camera = (self.active_camera != 0
            && entities.contains(&self.active_camera))
        .then_some(self.active_camera);
        let camera_index_lookups = u64::from(detached_active_camera.is_some());
        let prior_lifecycle_staging =
            std::mem::replace(&mut self.record_staged_lifecycle_events, true);
        let lifecycle_start = self.staged_lifecycle_events.len();
        let mut entries = Vec::with_capacity(entities.len());
        let mut swap_repairs = 0_u64;
        for entity in detach_preorder.iter().rev().copied() {
            let (entry, repaired_swap) = self.detach_preflighted_entity(entity);
            entries.push(entry);
            swap_repairs = swap_repairs.saturating_add(u64::from(repaired_swap));
        }
        entries.sort_unstable_by_key(|entry| entry.stable_order);

        if detached_active_camera.is_some() {
            self.active_camera = self.first_stable_camera_entity().unwrap_or(0);
        }
        self.bump_lifecycle_visibility_revision();
        self.mark_derived_state_dirty();
        self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        self.advance_world_generation();
        self.record_staged_lifecycle_events = prior_lifecycle_staging;
        if !prior_lifecycle_staging {
            for event in self.staged_lifecycle_events.split_off(lifecycle_start) {
                self.dispatch_component_lifecycle(event);
            }
        }

        let batch = DetachedEntityBatch {
            entries,
            restore_order,
            detached_active_camera,
        };
        let mut stats = batch.operation_stats();
        stats.swap_repairs = swap_repairs;
        stats.ordered_removals = stats.moved_rows;
        stats.hierarchy_index_lookups = (entities.len() + hierarchy_index_rebuild_rows) as u64;
        stats.camera_index_lookups = camera_index_lookups;
        self.record_detached_entity_batch_commit(stats);
        Ok(batch)
    }

    /// Restores a detached batch. On preflight failure the batch is returned
    /// untouched and this World remains unmodified.
    /// 预检失败仍会增加拒绝计数，原批次可用于撤销重试；通过后按预存父先子顺序恢复，再派发暂存的生命周期事件。
    pub fn restore_detached_entity_batch(
        &mut self,
        batch: DetachedEntityBatch,
    ) -> Result<(), DetachedEntityBatchRestoreError> {
        if let Err(error) = self.preflight_detached_batch(&batch) {
            self.record_detached_entity_batch_rejected_preflight();
            return Err(DetachedEntityBatchRestoreError { error, batch });
        }

        let mut stats = batch.operation_stats();
        stats.hierarchy_index_lookups = stats.moved_rows;
        stats.camera_index_lookups = u64::from(batch.detached_active_camera.is_some());

        let DetachedEntityBatch {
            entries,
            restore_order,
            detached_active_camera,
        } = batch;
        let prior_lifecycle_staging =
            std::mem::replace(&mut self.record_staged_lifecycle_events, true);
        let lifecycle_start = self.staged_lifecycle_events.len();
        let mut entries = entries.into_iter().map(Some).collect::<Vec<_>>();
        for index in restore_order {
            let entry = entries[index]
                .take()
                .expect("preflighted detached restore index must be unique");
            self.restore_preflighted_detached_entity(entry);
        }
        debug_assert!(entries.into_iter().all(|entry| entry.is_none()));
        if let Some(active_camera) = detached_active_camera {
            self.active_camera = active_camera;
        }
        self.bump_lifecycle_visibility_revision();
        self.mark_derived_state_dirty();
        self.inspection_artifact_cache.mark_hierarchy_rows_dirty();
        self.advance_world_generation();
        self.record_staged_lifecycle_events = prior_lifecycle_staging;
        if !prior_lifecycle_staging {
            for event in self.staged_lifecycle_events.split_off(lifecycle_start) {
                self.dispatch_component_lifecycle(event);
            }
        }
        self.record_detached_entity_batch_commit(stats);
        Ok(())
    }

    fn preflight_detached_entities(&self, entities: &[EntityId]) -> SceneResult<()> {
        for entity in entities {
            let location = self
                .internal_entity_location(*entity)
                .ok_or_else(|| SceneError::missing_entity("detach", *entity))?
                .location;
            let signature = self
                .archetype_index
                .signature(location.archetype_id)
                .ok_or(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity location is missing its archetype signature",
                })?;
            if self.kinds.get(entity).is_none() || self.stable_entity_order(*entity).is_none() {
                return Err(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity is missing identity metadata",
                });
            }
            for component_id in signature.sparse_set_components() {
                if !self.contains_component_id(*entity, *component_id) {
                    return Err(SceneError::DetachedEntityBatchInvariant {
                        reason: "detached entity is missing a sparse signature component",
                    });
                }
            }
        }
        Ok(())
    }

    fn detach_preflighted_entity(&mut self, entity: EntityId) -> (DetachedEntityBatchEntry, bool) {
        let stable_location = self
            .internal_entity_location(entity)
            .expect("preflighted detached entity must retain a location");
        let signature = self
            .archetype_index
            .signature(stable_location.location.archetype_id)
            .expect("preflighted detached entity must retain an archetype signature")
            .clone();
        let kind = *self
            .kinds
            .get(&entity)
            .expect("preflighted detached entity must retain a node kind");
        let stable_order = self
            .stable_entity_order(entity)
            .expect("preflighted detached entity must retain a stable order");
        let previous_parent = self.parent_of(entity);
        let repairs_swap = self
            .archetype_index
            .entities(stable_location.location.archetype_id)
            .is_some_and(|entities| stable_location.location.table_row + 1 < entities.len());
        let table_components = self.take_archetype_row_components(entity, stable_location.location);
        let sparse_components = self
            .component_storage
            .extract_entity_rows(stable_location.internal, signature.sparse_set_components());
        let dynamic_components = self.dynamic_components.remove(&entity);
        if let Some(dynamic_components) = dynamic_components.as_ref() {
            for component_id in dynamic_components.keys() {
                self.advance_dynamic_component_generation(component_id);
                self.invalidate_world_component_type(component_id);
            }
        }
        let observers = self.observers.detach_entity_observers(entity);

        self.remove_hierarchy_mutation_index_entry(entity, stable_order, previous_parent);

        for component_id in signature.ordered_component_ids() {
            self.trigger_component_lifecycle(LifecycleEventKind::Remove, entity, component_id);
            self.trigger_component_lifecycle(LifecycleEventKind::Despawn, entity, component_id);
            if let Some((type_id, type_name)) =
                self.component_registry.rust_type_for_id(component_id)
            {
                self.removed_component_events
                    .push_type_id(type_id, type_name, entity);
            }
        }
        self.unregister_stable_entity(entity);
        let removed = self.remove_entity_from_dense_storage(entity);
        debug_assert!(removed);
        let removed_kind = self
            .kinds
            .remove(&entity)
            .expect("preflighted detached entity must retain a node kind");
        debug_assert_eq!(removed_kind, kind);
        self.record_node_kind_removed(kind);
        self.inspection_artifact_cache.mark_fields_dirty(entity);
        self.advance_scene_binding_generations_for_removal(entity, previous_parent);
        self.record_world_fact(zircon_runtime_interface::world_sync::WorldFact::Despawned(
            entity,
        ));

        (
            DetachedEntityBatchEntry {
                entity,
                kind,
                stable_order,
                signature,
                table_components,
                sparse_components,
                dynamic_components,
                observers,
            },
            repairs_swap,
        )
    }

    fn preflight_detached_batch(&self, batch: &DetachedEntityBatch) -> SceneResult<()> {
        if batch.entries.is_empty() {
            return Err(SceneError::DetachedEntityBatchInvariant {
                reason: "detached entity batch is empty",
            });
        }
        if batch.restore_order.len() != batch.entries.len() {
            return Err(SceneError::DetachedEntityBatchInvariant {
                reason: "detached entity batch restore plan has the wrong length",
            });
        }
        let mut restore_indices = BTreeSet::new();
        for index in batch.restore_order.iter().copied() {
            if index >= batch.entries.len() || !restore_indices.insert(index) {
                return Err(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch restore plan is not a permutation",
                });
            }
        }
        let mut entity_ids = BTreeSet::new();
        let mut stable_orders = BTreeSet::new();
        for entry in &batch.entries {
            if !entity_ids.insert(entry.entity) || !stable_orders.insert(entry.stable_order) {
                return Err(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch contains duplicate identity metadata",
                });
            }
        }
        self.entity_registry
            .ensure_capacity_for_additional(entity_ids.len())?;
        let hierarchy_component_id = self.registered_component_id::<Hierarchy>();
        let mut batch_parents = HashMap::with_capacity(batch.entries.len());
        for entry in &batch.entries {
            if self.contains_entity(entry.entity)
                || self.entity_registry.contains_stable(entry.entity)
            {
                return Err(SceneError::DuplicateEntity {
                    entity: entry.entity,
                });
            }
            if self.stable_entity_order_is_occupied(entry.stable_order) {
                return Err(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch stable order is already occupied",
                });
            }
            let archetype_id = self
                .archetype_index
                .id_for_signature(&entry.signature)
                .ok_or(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch archetype no longer exists",
                })?;
            self.archetype_index
                .validate_row_components(archetype_id, &entry.table_components)
                .map_err(|_| SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch table row no longer matches its archetype",
                })?;
            for sparse in &entry.sparse_components {
                self.component_storage
                    .validate_transferred_row(sparse.component_id(), sparse)?;
            }
            if let Some(hierarchy_component_id) = hierarchy_component_id {
                let hierarchy = entry
                    .table_components
                    .get(&hierarchy_component_id)
                    .and_then(|(value, _)| value.downcast_ref::<Hierarchy>())
                    .ok_or(SceneError::DetachedEntityBatchInvariant {
                        reason: "detached entity batch is missing its hierarchy boundary",
                    })?;
                batch_parents.insert(entry.entity, hierarchy.parent);
                if let Some(parent) = hierarchy.parent {
                    if !self.contains_entity(parent) && !entity_ids.contains(&parent) {
                        return Err(SceneError::MissingParent {
                            child: entry.entity,
                            parent,
                        });
                    }
                }
            } else {
                batch_parents.insert(entry.entity, None);
            }
        }
        if let Some(detached_active_camera) = batch.detached_active_camera {
            let active_camera_is_valid = if self.contains_entity(detached_active_camera) {
                self.contains_component::<CameraComponent>(detached_active_camera)
            } else {
                self.registered_component_id::<CameraComponent>()
                    .is_some_and(|camera_component_id| {
                        batch.entries.iter().any(|entry| {
                            entry.entity == detached_active_camera
                                && entry.signature.contains(camera_component_id)
                        })
                    })
            };
            if !active_camera_is_valid {
                return Err(SceneError::DetachedEntityBatchInvariant {
                    reason: "detached entity batch active camera is no longer available",
                });
            }
        }
        restore_parent_validation::validate_restore_parent_chains(
            &batch_parents,
            batch.entries.iter().map(|entry| entry.entity),
            |entity| self.parent_of(entity),
        )?;
        Ok(())
    }

    fn restore_preflighted_detached_entity(&mut self, entry: DetachedEntityBatchEntry) {
        let archetype_id = self
            .archetype_index
            .id_for_signature(&entry.signature)
            .expect("preflighted detached batch archetype must remain registered");
        let internal =
            self.register_prevalidated_stable_entity_without_row(entry.entity, entry.stable_order);
        self.append_entity_to_dense_storage(entry.entity);
        let previous = self.kinds.insert(entry.entity, entry.kind);
        debug_assert!(previous.is_none());
        self.record_node_kind_added(entry.kind);
        self.append_entity_archetype_row(entry.entity, archetype_id, entry.table_components);
        let restored_parent = self.parent_of(entry.entity);
        self.update_hierarchy_mutation_index(entry.entity, None, restored_parent);
        for sparse in entry.sparse_components {
            let component_id = sparse.component_id();
            let preflight = self
                .component_storage
                .preflight_transferred_row(component_id, sparse)
                .expect("preflighted detached sparse row must remain compatible");
            let replaced = self
                .component_storage
                .restore_preflighted_transferred_row(internal, preflight);
            debug_assert!(!replaced);
        }
        if let Some(dynamic_components) = entry.dynamic_components {
            for component_id in dynamic_components.keys() {
                self.advance_dynamic_component_generation(component_id);
                self.invalidate_world_component_type(component_id);
            }
            let previous = self
                .dynamic_components
                .insert(entry.entity, dynamic_components);
            debug_assert!(previous.is_none());
        }
        self.observers
            .restore_detached_entity_observers(entry.observers);
        self.inspection_artifact_cache
            .mark_fields_dirty(entry.entity);
        for component_id in entry.signature.ordered_component_ids() {
            self.trigger_component_lifecycle(LifecycleEventKind::Add, entry.entity, component_id);
            self.trigger_component_lifecycle(
                LifecycleEventKind::Insert,
                entry.entity,
                component_id,
            );
        }
        self.advance_scene_binding_generations_for_new_descendant(entry.entity);
        self.record_world_fact(zircon_runtime_interface::world_sync::WorldFact::Spawned(
            entry.entity,
        ));
    }
}
