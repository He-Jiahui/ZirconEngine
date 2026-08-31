use std::collections::BTreeMap;

use crate::scene::ecs::StorageType;
use zircon_runtime_interface::world_sync::WorldFact;

use super::{
    BundleInsertionTransaction, MAX_BUNDLE_COMPONENT_TYPES, PendingBundleEffect,
    PendingBundlePublication, PendingDeferredRemoval, TableBundlePublication,
};

impl BundleInsertionTransaction<'_> {
    pub(crate) fn finish_with_deferred_despawn(mut self) -> crate::scene::SceneResult<()> {
        match self.target.take() {
            Some(super::BundleTarget::Spawn(_)) => Ok(()),
            Some(super::BundleTarget::Existing(_)) => {
                self.world.preflight_deferred_despawn(self.entity)?;
                let lifecycle_start = self.world.staged_lifecycle_events.len();
                let prior_lifecycle_staging =
                    std::mem::replace(&mut self.world.record_staged_lifecycle_events, true);
                let result = self.world.remove_entity(self.entity);
                self.world.record_staged_lifecycle_events = prior_lifecycle_staging;
                if result.is_err() {
                    self.world.staged_lifecycle_events.truncate(lifecycle_start);
                    return result;
                }
                if prior_lifecycle_staging {
                    return Ok(());
                }
                let lifecycle_events = self
                    .world
                    .staged_lifecycle_events
                    .split_off(lifecycle_start);
                for event in lifecycle_events {
                    self.world.dispatch_component_lifecycle(event);
                }
                Ok(())
            }
            None => Err(crate::scene::SceneError::BundleTransactionInvariant {
                reason: "pending entity target is missing",
            }),
        }
    }

    pub(crate) fn finish_with_deferred_spawn(
        mut self,
        deferred_spawn: Option<crate::scene::ecs::DeferredSpawnToken>,
    ) -> crate::scene::SceneResult<()> {
        if self.defer_final_state_validation {
            self.defer_final_state_validation = false;
            self.validate_final_state()?;
        }
        let spawns_entity = matches!(self.target.as_ref(), Some(super::BundleTarget::Spawn(_)));
        if (self.component_count != 0 || self.deferred_removal_count != 0 || spawns_entity)
            && !self.final_state_validated.get()
        {
            return Err(crate::scene::SceneError::BundleFinalStateNotValidated);
        }
        if self.component_count == 0 && self.deferred_removal_count == 0 && !spawns_entity {
            return Ok(());
        }

        // Detach every fallible input before any descriptor or entity map changes.
        self.validate_commit_invariants()?;
        let prepared_values = self.take_prepared_values()?;
        let default_values = self.take_default_values()?;
        let deferred_removals = self.deferred_removals;
        let deferred_removal_count = self.deferred_removal_count;
        let final_signature = self.final_archetype_signature(
            spawns_entity,
            &default_values,
            &prepared_values,
            &deferred_removals,
            deferred_removal_count,
        );
        let commit_input = self.prepare_commit()?;
        let hierarchy_parent_before = self.world.parent_of(self.entity);
        self.materialize_reserved_component_types();
        let archetype_assignments_before = self.world.archetype_assignment_count();
        let boundary = self.begin_commit(commit_input);
        let staged_value_allocations = self.default_value_count + self.component_count;
        let tick = self.world.mutation_change_tick();
        let mut table_values: [Option<TableBundlePublication>; MAX_BUNDLE_COMPONENT_TYPES] =
            std::array::from_fn(|_| None);
        let mut table_value_count = 0_usize;
        let mut effects: [Option<PendingBundleEffect>; MAX_BUNDLE_COMPONENT_TYPES] =
            [None; MAX_BUNDLE_COMPONENT_TYPES];
        let mut effect_count = 0_usize;

        let mut component_storage_moves = 0;
        for default_value in default_values
            .into_iter()
            .take(self.default_value_count)
            .flatten()
        {
            if prepared_values
                .iter()
                .flatten()
                .any(|prepared| prepared.preflight.type_id == default_value.type_id())
            {
                continue;
            }
            let was_present = self
                .world
                .contains_component_id(self.entity, default_value.component_id());
            effects[effect_count] =
                Some(default_value.prepare_effect(was_present, hierarchy_parent_before));
            effect_count += 1;
            match default_value.publish_value(&mut *self.world, boundary.internal, tick) {
                PendingBundlePublication::Table(value) => {
                    table_values[table_value_count] = Some(value);
                    table_value_count += 1;
                }
                PendingBundlePublication::Sparse { replaced } => {
                    debug_assert_eq!(replaced, was_present);
                }
            }
            component_storage_moves += 1;
        }
        for prepared in prepared_values
            .into_iter()
            .take(self.component_count)
            .flatten()
        {
            debug_assert!(matches!(
                self.world
                    .component_registry
                    .rust_type_for_id(prepared.preflight.component_id),
                Some((type_id, _)) if type_id == prepared.preflight.type_id
            ));
            let was_present = self
                .world
                .contains_component_id(self.entity, prepared.preflight.component_id);
            effects[effect_count] = Some(
                prepared
                    .component
                    .prepare_effect(was_present, hierarchy_parent_before),
            );
            effect_count += 1;
            match prepared
                .component
                .publish_value(&mut *self.world, boundary.internal, tick)
            {
                PendingBundlePublication::Table(value) => {
                    table_values[table_value_count] = Some(value);
                    table_value_count += 1;
                }
                PendingBundlePublication::Sparse { replaced } => {
                    debug_assert_eq!(replaced, was_present);
                }
            }
            component_storage_moves += 1;
        }

        let current_signature = self
            .world
            .entity_archetype_signature(self.entity)
            .expect("bundle target must own an archetype row at publication");
        let final_archetype_transition = current_signature != final_signature;
        let mut table_removals: [Option<PendingDeferredRemoval>; MAX_BUNDLE_COMPONENT_TYPES] =
            [None; MAX_BUNDLE_COMPONENT_TYPES];
        let mut table_removal_count = 0_usize;
        let mut sparse_removals: [Option<PendingDeferredRemoval>; MAX_BUNDLE_COMPONENT_TYPES] =
            [None; MAX_BUNDLE_COMPONENT_TYPES];
        let mut sparse_removal_count = 0_usize;
        for removal in deferred_removals
            .into_iter()
            .take(deferred_removal_count)
            .flatten()
        {
            match removal.storage_type() {
                StorageType::Table => {
                    table_removals[table_removal_count] = Some(removal);
                    table_removal_count += 1;
                }
                StorageType::SparseSet => {
                    sparse_removals[sparse_removal_count] = Some(removal);
                    sparse_removal_count += 1;
                }
            }
        }
        if final_archetype_transition {
            let mut updates = table_values
                .into_iter()
                .take(table_value_count)
                .flatten()
                .map(|publication| {
                    (
                        publication.component_id,
                        Some((publication.value, publication.ticks)),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            for removal in table_removals.iter().take(table_removal_count).flatten() {
                updates.insert(removal.component_id(), None);
            }
            let mut transition =
                self.world
                    .transition_entity_archetype_row(self.entity, final_signature, updates);
            let previous = transition
                .as_mut()
                .expect("a changed deferred final signature must transition one row");
            for removal in table_removals
                .into_iter()
                .take(table_removal_count)
                .flatten()
            {
                let (value, _) = previous
                    .remove(&removal.component_id())
                    .expect("preflighted deferred table removal must take its old value");
                removal.publish_table(&mut *self.world, self.entity, value);
            }
        } else {
            debug_assert_eq!(table_removal_count, 0);
            let location = self
                .world
                .internal_entity_location(self.entity)
                .expect("bundle target must retain its archetype row")
                .location;
            for publication in table_values.into_iter().take(table_value_count).flatten() {
                let replaced = self.world.archetype_index.replace(
                    location.archetype_id,
                    location.table_row,
                    publication.component_id,
                    publication.value,
                    tick,
                );
                debug_assert!(replaced.is_some());
            }
        }
        for removal in sparse_removals
            .into_iter()
            .take(sparse_removal_count)
            .flatten()
        {
            let removed = removal.publish_sparse(&mut *self.world, self.entity, boundary.internal);
            debug_assert!(removed);
        }
        component_storage_moves += table_removal_count + sparse_removal_count;
        if let Some(current_parent) = self.staged_hierarchy_parent {
            self.world.update_hierarchy_mutation_index(
                self.entity,
                hierarchy_parent_before,
                current_parent,
            );
        }
        for effect in effects.into_iter().take(effect_count).flatten() {
            effect.apply(&mut *self.world, self.entity);
        }
        if final_archetype_transition {
            self.world.bump_lifecycle_visibility_revision();
        }
        if boundary.spawned_entity {
            self.world.mark_derived_state_dirty();
            self.world
                .inspection_artifact_cache
                .mark_hierarchy_rows_dirty();
            self.world
                .advance_scene_binding_generations_for_new_descendant(self.entity);
        }
        self.world.advance_world_generation();
        if boundary.spawned_entity {
            self.world
                .record_world_fact(WorldFact::Spawned(self.entity));
            if let Some(token) = deferred_spawn {
                self.world.mark_deferred_spawn_published(token);
            }
        }
        let lifecycle_events = self
            .world
            .staged_lifecycle_events
            .len()
            .saturating_sub(boundary.lifecycle_start);
        let archetype_assignments = self
            .world
            .archetype_assignment_count()
            .saturating_sub(archetype_assignments_before);
        self.world.record_bundle_transaction_diagnostics(
            final_archetype_transition,
            archetype_assignments,
            component_storage_moves,
            lifecycle_events,
            staged_value_allocations,
        );
        self.world.record_staged_lifecycle_events = boundary.prior_lifecycle_staging;
        if boundary.prior_lifecycle_staging {
            return Ok(());
        }
        let lifecycle_events = self
            .world
            .staged_lifecycle_events
            .split_off(boundary.lifecycle_start);
        for event in lifecycle_events {
            self.world.dispatch_component_lifecycle(event);
        }
        Ok(())
    }
}
