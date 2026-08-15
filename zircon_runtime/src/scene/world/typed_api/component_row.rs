use std::collections::BTreeMap;

use crate::scene::components::Hierarchy;
use crate::scene::ecs::{
    ArchetypeSignature, Component, ComponentId, ComponentStorage, ComponentTicks, InternalEntity,
    LifecycleEventKind, PreflightedComponentInsert, StorageType,
};
use crate::scene::{EntityId, World};

type DenseComponentValue = Box<dyn std::any::Any + Send + Sync>;

pub(in crate::scene::world) struct PendingComponentRow {
    signature: ArchetypeSignature,
    tick: crate::scene::ecs::ChangeTick,
    dense_updates: BTreeMap<ComponentId, Option<(DenseComponentValue, ComponentTicks)>>,
    sparse_values: BTreeMap<ComponentId, Box<dyn PendingSparseValue>>,
    initially_present: BTreeMap<ComponentId, bool>,
}

impl PendingComponentRow {
    fn new(signature: ArchetypeSignature, tick: crate::scene::ecs::ChangeTick) -> Self {
        Self {
            signature,
            tick,
            dense_updates: BTreeMap::new(),
            sparse_values: BTreeMap::new(),
            initially_present: BTreeMap::new(),
        }
    }
}

trait PendingSparseValue {
    fn publish(
        self: Box<Self>,
        storage: &mut ComponentStorage,
        entity: InternalEntity,
        tick: crate::scene::ecs::ChangeTick,
    ) -> bool;
}

struct TypedPendingSparseValue<T> {
    preflight: PreflightedComponentInsert<T>,
    value: T,
}

impl<T> PendingSparseValue for TypedPendingSparseValue<T>
where
    T: Component,
{
    fn publish(
        self: Box<Self>,
        storage: &mut ComponentStorage,
        entity: InternalEntity,
        tick: crate::scene::ecs::ChangeTick,
    ) -> bool {
        let Self { preflight, value } = *self;
        storage.insert_preflighted_at_tick(preflight, entity, value, tick)
    }
}

impl World {
    pub(in crate::scene::world) fn begin_component_row(
        &mut self,
        entity: EntityId,
    ) -> PendingComponentRow {
        let signature = self
            .entity_archetype_signature(entity)
            .expect("component row target must own an archetype signature");
        PendingComponentRow::new(signature, self.mutation_change_tick())
    }

    pub(in crate::scene::world) fn begin_empty_component_row(&mut self) -> PendingComponentRow {
        PendingComponentRow::new(ArchetypeSignature::empty(), self.mutation_change_tick())
    }

    pub(in crate::scene::world) fn stage_component_row_value<T>(
        &mut self,
        row: &mut PendingComponentRow,
        component: T,
    ) where
        T: Component,
    {
        let component_id = self.component_id::<T>();
        self.stage_component_row_value_with_id(row, component_id, component);
        self.mark_component_derived_state_dirty::<T>();
    }

    pub(in crate::scene::world) fn stage_component_row_value_with_id<T>(
        &mut self,
        row: &mut PendingComponentRow,
        component_id: ComponentId,
        component: T,
    ) where
        T: Component,
    {
        let initially_present = row.signature.contains(component_id);
        row.initially_present
            .entry(component_id)
            .or_insert(initially_present);
        row.signature = row
            .signature
            .with_component_added(component_id, T::STORAGE_TYPE);
        match T::STORAGE_TYPE {
            StorageType::Table => {
                row.dense_updates.insert(
                    component_id,
                    Some((Box::new(component), ComponentTicks::new(row.tick))),
                );
            }
            StorageType::SparseSet => {
                let preflight = self
                    .component_storage
                    .preflight_insert::<T>(component_id, StorageType::SparseSet)
                    .expect("prevalidated sparse row value must preserve its component schema");
                row.sparse_values.insert(
                    component_id,
                    Box::new(TypedPendingSparseValue {
                        preflight,
                        value: component,
                    }),
                );
            }
        }
    }

    pub(in crate::scene::world) fn commit_component_row(
        &mut self,
        entity: EntityId,
        row: PendingComponentRow,
        emit_lifecycle: bool,
    ) -> bool {
        let PendingComponentRow {
            signature,
            tick,
            dense_updates,
            sparse_values,
            initially_present,
        } = row;
        let hierarchy_update =
            self.registered_component_id::<Hierarchy>()
                .and_then(|component_id| {
                    dense_updates.get(&component_id).map(|value| {
                        let current_parent = value
                            .as_ref()
                            .and_then(|(value, _)| value.downcast_ref::<Hierarchy>())
                            .and_then(|hierarchy| hierarchy.parent);
                        (self.parent_of(entity), current_parent)
                    })
                });
        let internal = self
            .internal_entity(entity)
            .expect("component row target must retain its registered identity");
        let current_signature = self
            .entity_archetype_signature(entity)
            .expect("component row target must retain its source archetype signature");
        let target_archetype = self.ensure_archetype(signature.clone());
        self.archetype_index
            .validate_transition(
                target_archetype,
                current_signature.table_components().iter().copied(),
                &dense_updates,
            )
            .expect("staged component row must match its final archetype schema");

        for (component_id, sparse) in sparse_values {
            let replaced = sparse.publish(&mut self.component_storage, internal, tick);
            debug_assert_eq!(
                replaced,
                initially_present
                    .get(&component_id)
                    .copied()
                    .unwrap_or(false)
            );
        }

        let transitioned = current_signature != signature;
        if transitioned {
            let previous = self.transition_entity_archetype_row(entity, signature, dense_updates);
            debug_assert!(previous.is_some());
        } else {
            let location = self
                .internal_entity_location(entity)
                .expect("component row target must retain its dense location")
                .location;
            for (component_id, value) in dense_updates {
                let Some((value, _)) = value else {
                    continue;
                };
                let replaced = self.archetype_index.replace(
                    location.archetype_id,
                    location.table_row,
                    component_id,
                    value,
                    tick,
                );
                debug_assert!(replaced.is_some());
            }
        }

        if let Some((previous_parent, current_parent)) = hierarchy_update {
            self.update_hierarchy_mutation_index(entity, previous_parent, current_parent);
        }
        if emit_lifecycle {
            for (component_id, was_present) in initially_present {
                let kind = if was_present {
                    LifecycleEventKind::Replace
                } else {
                    LifecycleEventKind::Add
                };
                self.trigger_component_lifecycle(kind, entity, component_id);
                self.trigger_component_lifecycle(LifecycleEventKind::Insert, entity, component_id);
            }
        }
        transitioned
    }

    pub(in crate::scene::world) fn commit_rebuilt_component_row(
        &mut self,
        entity: EntityId,
        row: PendingComponentRow,
    ) {
        let PendingComponentRow {
            signature,
            tick,
            dense_updates,
            sparse_values,
            initially_present,
        } = row;
        debug_assert!(initially_present.values().all(|present| !present));
        let internal = self
            .internal_entity(entity)
            .expect("projection row target must retain its rebuilt identity");
        let target_archetype = self.ensure_archetype(signature);
        self.archetype_index
            .validate_transition(target_archetype, std::iter::empty(), &dense_updates)
            .expect("rebuilt component row must match its final archetype schema");

        for (_, sparse) in sparse_values {
            let replaced = sparse.publish(&mut self.component_storage, internal, tick);
            debug_assert!(!replaced);
        }

        let mut dense_components = BTreeMap::new();
        for (component_id, value) in dense_updates {
            let value = value.expect("rebuilt component rows only stage inserted dense values");
            dense_components.insert(component_id, value);
        }
        self.append_entity_archetype_row(entity, target_archetype, dense_components);
    }
}
