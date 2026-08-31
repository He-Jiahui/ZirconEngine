use std::any::TypeId;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;
use std::marker::PhantomData;

use crate::scene::ecs::{ChangeTick, ComponentId, ComponentTicks, InternalEntity, StorageType};

use super::super::{ComponentRemoveResult, StorageError};
use super::component_results::downcast_component;
use super::entry::{PreflightedTransferredComponentRow, StoredComponent, TransferredComponentRow};
use super::location::ComponentStorageLocation;
use super::sparse::SparseComponentStorage;

/// Owns sparse-set values only. Dense table values live in `ArchetypeTable`.
#[derive(Default)]
pub(crate) struct ComponentStorage {
    storage_types: HashMap<ComponentId, StorageType>,
    component_types: HashMap<ComponentId, TypeId>,
    sparse_components: HashMap<ComponentId, SparseComponentStorage>,
}

/// Proves that a component insert's id and representation were validated
/// before a structural transaction publishes its entity row.
pub(crate) struct PreflightedComponentInsert<T> {
    component_id: ComponentId,
    storage_type: StorageType,
    marker: PhantomData<fn() -> T>,
}

impl<T> PreflightedComponentInsert<T> {
    pub(crate) fn component_id(&self) -> ComponentId {
        self.component_id
    }
}

impl ComponentStorage {
    pub(crate) fn transferred_table_row(
        component_id: ComponentId,
        source_ticks: ComponentTicks,
        value: StoredComponent,
    ) -> TransferredComponentRow {
        let type_id = value.as_ref().type_id();
        TransferredComponentRow {
            component_id,
            storage_type: StorageType::Table,
            type_id,
            source_ticks,
            value,
        }
    }

    pub(crate) fn preflighted_transferred_storage_type(
        row: &PreflightedTransferredComponentRow,
    ) -> StorageType {
        row.row.storage_type
    }

    pub(crate) fn take_preflighted_transferred_value(
        row: PreflightedTransferredComponentRow,
    ) -> StoredComponent {
        row.row.value
    }

    pub(crate) fn insert<T>(
        &mut self,
        component_id: ComponentId,
        storage_type: StorageType,
        entity: InternalEntity,
        value: T,
    ) -> Result<Option<T>, StorageError>
    where
        T: 'static + Send + Sync,
    {
        self.insert_at_tick(
            component_id,
            storage_type,
            entity,
            value,
            ChangeTick::INITIAL,
        )
    }

    pub(crate) fn insert_at_tick<T>(
        &mut self,
        component_id: ComponentId,
        storage_type: StorageType,
        entity: InternalEntity,
        value: T,
        tick: ChangeTick,
    ) -> Result<Option<T>, StorageError>
    where
        T: 'static + Send + Sync,
    {
        self.ensure_sparse_storage_type(component_id, storage_type)?;
        self.ensure_component_type::<T>(component_id)?;
        let old = self
            .sparse_components
            .entry(component_id)
            .or_default()
            .insert(entity, Box::new(value), tick);
        let Some(old) = old else {
            return Ok(None);
        };
        Ok(Some(downcast_component(component_id, old)?))
    }

    pub(crate) fn insert_preflighted_at_tick<T>(
        &mut self,
        preflight: PreflightedComponentInsert<T>,
        entity: InternalEntity,
        value: T,
        tick: ChangeTick,
    ) -> bool
    where
        T: 'static + Send + Sync,
    {
        let PreflightedComponentInsert {
            component_id,
            storage_type,
            marker: _,
        } = preflight;
        assert_eq!(
            storage_type,
            StorageType::SparseSet,
            "dense bundle values must publish through ArchetypeTable"
        );
        self.storage_types
            .entry(component_id)
            .or_insert(StorageType::SparseSet);
        self.component_types
            .entry(component_id)
            .or_insert(TypeId::of::<T>());
        self.sparse_components
            .entry(component_id)
            .or_default()
            .insert(entity, Box::new(value), tick)
            .is_some()
    }

    pub(crate) fn validate_insert<T>(
        &self,
        component_id: ComponentId,
        storage_type: StorageType,
    ) -> Result<(), StorageError>
    where
        T: 'static + Send + Sync,
    {
        if storage_type == StorageType::SparseSet {
            if let Some(existing) = self.storage_types.get(&component_id).copied() {
                if existing != storage_type {
                    return Err(StorageError::StorageTypeMismatch {
                        component_id,
                        existing,
                        requested: storage_type,
                    });
                }
            }
            if let Some(existing) = self.component_types.get(&component_id) {
                if *existing != TypeId::of::<T>() {
                    return Err(StorageError::ComponentTypeMismatch { component_id });
                }
            }
        }
        Ok(())
    }

    pub(crate) fn preflight_insert<T>(
        &self,
        component_id: ComponentId,
        storage_type: StorageType,
    ) -> Result<PreflightedComponentInsert<T>, StorageError>
    where
        T: 'static + Send + Sync,
    {
        self.validate_insert::<T>(component_id, storage_type)?;
        Ok(PreflightedComponentInsert {
            component_id,
            storage_type,
            marker: PhantomData,
        })
    }

    pub(crate) fn get<T>(&self, component_id: ComponentId, entity: InternalEntity) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        self.sparse_components.get(&component_id)?.get(entity)
    }

    pub(crate) fn get_mut<T>(
        &mut self,
        component_id: ComponentId,
        entity: InternalEntity,
    ) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        self.sparse_components
            .get_mut(&component_id)?
            .get_mut(entity)
    }

    pub(crate) fn get_mut_at_tick<T>(
        &mut self,
        component_id: ComponentId,
        entity: InternalEntity,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        self.sparse_components
            .get_mut(&component_id)?
            .get_mut_at_tick(entity, tick)
    }

    pub(crate) fn get_mut_with_ticks<T>(
        &mut self,
        component_id: ComponentId,
        entity: InternalEntity,
    ) -> Option<(&mut T, &mut ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        self.sparse_components
            .get_mut(&component_id)?
            .get_mut_with_ticks(entity)
    }

    pub(crate) fn remove<T>(
        &mut self,
        component_id: ComponentId,
        entity: InternalEntity,
    ) -> Result<Option<ComponentRemoveResult<T>>, StorageError>
    where
        T: 'static + Send + Sync,
    {
        self.ensure_component_type::<T>(component_id)?;
        let Some(removed) = self
            .sparse_components
            .get_mut(&component_id)
            .and_then(|storage| storage.remove(entity))
        else {
            return Ok(None);
        };
        Ok(Some(ComponentRemoveResult {
            value: downcast_component(component_id, removed.value)?,
        }))
    }

    pub(crate) fn extract_entity_rows(
        &mut self,
        entity: InternalEntity,
        component_ids: &[ComponentId],
    ) -> Vec<TransferredComponentRow> {
        let mut rows = Vec::with_capacity(component_ids.len());
        for component_id in component_ids {
            let Some(type_id) = self.component_types.get(component_id).copied() else {
                continue;
            };
            let Some(removed) = self
                .sparse_components
                .get_mut(component_id)
                .and_then(|storage| storage.remove(entity))
            else {
                continue;
            };
            rows.push(TransferredComponentRow {
                component_id: *component_id,
                storage_type: StorageType::SparseSet,
                type_id,
                source_ticks: removed.ticks,
                value: removed.value,
            });
        }
        rows
    }

    pub(crate) fn insert_transferred_row(
        &mut self,
        component_id: ComponentId,
        entity: InternalEntity,
        row: TransferredComponentRow,
        tick: ChangeTick,
    ) -> Result<bool, StorageError> {
        let preflight = self.preflight_transferred_row(component_id, row)?;
        Ok(self.insert_preflighted_transferred_row(entity, preflight, tick))
    }

    pub(crate) fn preflight_transferred_row(
        &self,
        component_id: ComponentId,
        mut row: TransferredComponentRow,
    ) -> Result<PreflightedTransferredComponentRow, StorageError> {
        self.validate_transferred_row(component_id, &row)?;
        row.component_id = component_id;
        Ok(PreflightedTransferredComponentRow { component_id, row })
    }

    pub(crate) fn validate_transferred_row(
        &self,
        component_id: ComponentId,
        row: &TransferredComponentRow,
    ) -> Result<(), StorageError> {
        if row.value.as_ref().type_id() != row.type_id {
            return Err(StorageError::ComponentTypeMismatch { component_id });
        }
        if row.storage_type == StorageType::SparseSet {
            if let Some(existing) = self.storage_types.get(&component_id).copied() {
                if existing != row.storage_type {
                    return Err(StorageError::StorageTypeMismatch {
                        component_id,
                        existing,
                        requested: row.storage_type,
                    });
                }
            }
            if let Some(existing) = self.component_types.get(&component_id) {
                if *existing != row.type_id {
                    return Err(StorageError::ComponentTypeMismatch { component_id });
                }
            }
        }
        Ok(())
    }

    pub(crate) fn insert_preflighted_transferred_row(
        &mut self,
        entity: InternalEntity,
        preflight: PreflightedTransferredComponentRow,
        tick: ChangeTick,
    ) -> bool {
        let PreflightedTransferredComponentRow { component_id, row } = preflight;
        assert_eq!(
            row.storage_type,
            StorageType::SparseSet,
            "dense transferred values must publish through ArchetypeTable"
        );
        self.storage_types
            .entry(component_id)
            .or_insert(StorageType::SparseSet);
        self.component_types
            .entry(component_id)
            .or_insert(row.type_id);
        self.sparse_components
            .entry(component_id)
            .or_default()
            .insert(entity, row.value, tick)
            .is_some()
    }

    pub(crate) fn restore_preflighted_transferred_row(
        &mut self,
        entity: InternalEntity,
        preflight: PreflightedTransferredComponentRow,
    ) -> bool {
        let PreflightedTransferredComponentRow { component_id, row } = preflight;
        assert_eq!(
            row.storage_type,
            StorageType::SparseSet,
            "dense transferred values must publish through ArchetypeTable"
        );
        self.storage_types
            .entry(component_id)
            .or_insert(StorageType::SparseSet);
        self.component_types
            .entry(component_id)
            .or_insert(row.type_id);
        self.sparse_components
            .entry(component_id)
            .or_default()
            .insert_with_ticks(entity, row.value, row.source_ticks)
            .is_some()
    }

    pub(crate) fn contains(&self, component_id: ComponentId, entity: InternalEntity) -> bool {
        self.sparse_components
            .get(&component_id)
            .is_some_and(|storage| storage.contains(entity))
    }

    pub(crate) fn ticks(
        &self,
        component_id: ComponentId,
        entity: InternalEntity,
    ) -> Option<ComponentTicks> {
        self.sparse_components.get(&component_id)?.ticks(entity)
    }

    pub(crate) fn location(
        &self,
        component_id: ComponentId,
        entity: InternalEntity,
    ) -> Option<ComponentStorageLocation> {
        self.sparse_components
            .get(&component_id)?
            .contains(entity)
            .then_some(ComponentStorageLocation::sparse(component_id, entity))
    }

    pub(crate) fn get_with_ticks_at_location<T>(
        &self,
        location: ComponentStorageLocation,
    ) -> Option<(&T, ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        if location.storage_type != StorageType::SparseSet
            || location.table_row.is_some()
            || location.table_archetype.is_some()
            || location.table_column_slot.is_some()
        {
            return None;
        }
        self.sparse_components
            .get(&location.component_id)?
            .get_with_ticks(location.entity)
    }

    pub(crate) fn mark_changed(
        &mut self,
        component_id: ComponentId,
        entity: InternalEntity,
        tick: ChangeTick,
    ) {
        if let Some(storage) = self.sparse_components.get_mut(&component_id) {
            storage.mark_changed(entity, tick);
        }
    }

    pub(crate) fn remove_entity_components(
        &mut self,
        entity: InternalEntity,
        component_ids: &[ComponentId],
    ) -> Vec<ComponentId> {
        let mut removed = Vec::with_capacity(component_ids.len());
        for component_id in component_ids {
            if self
                .sparse_components
                .get_mut(component_id)
                .and_then(|storage| storage.remove(entity))
                .is_some()
            {
                removed.push(*component_id);
            }
        }
        removed
    }

    pub(crate) fn storage_type(&self, component_id: ComponentId) -> Option<StorageType> {
        self.storage_types.get(&component_id).copied()
    }

    pub(crate) fn len_for_component(&self, component_id: ComponentId) -> usize {
        self.sparse_components
            .get(&component_id)
            .map_or(0, SparseComponentStorage::len)
    }

    pub(crate) fn for_each_sparse_entity(
        &self,
        component_id: ComponentId,
        visit: impl FnMut(InternalEntity),
    ) {
        if let Some(storage) = self.sparse_components.get(&component_id) {
            storage.for_each_entity(visit);
        }
    }

    fn ensure_sparse_storage_type(
        &mut self,
        component_id: ComponentId,
        requested: StorageType,
    ) -> Result<(), StorageError> {
        if requested == StorageType::Table {
            return Err(StorageError::TableOwnedByArchetype { component_id });
        }
        match self.storage_types.entry(component_id) {
            Entry::Occupied(entry) => {
                let existing = *entry.get();
                if existing != requested {
                    return Err(StorageError::StorageTypeMismatch {
                        component_id,
                        existing,
                        requested,
                    });
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(requested);
            }
        }
        Ok(())
    }

    fn ensure_component_type<T>(&mut self, component_id: ComponentId) -> Result<(), StorageError>
    where
        T: 'static + Send + Sync,
    {
        let requested = TypeId::of::<T>();
        match self.component_types.entry(component_id) {
            Entry::Occupied(entry) => {
                if *entry.get() != requested {
                    return Err(StorageError::ComponentTypeMismatch { component_id });
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(requested);
            }
        }
        Ok(())
    }
}

impl fmt::Debug for ComponentStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut storage_types = self.storage_types.iter().collect::<Vec<_>>();
        storage_types.sort_by_key(|(component_id, _)| **component_id);
        f.debug_struct("ComponentStorage")
            .field("storage_types", &storage_types)
            .finish()
    }
}

impl Clone for ComponentStorage {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for ComponentStorage {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct SparseValue(u32);

    #[test]
    fn sparse_rows_rekey_at_the_target_tick() {
        let component_id = ComponentId::new(1);
        let source_entity = InternalEntity::new(7, 1);
        let target_entity = InternalEntity::new(29, 3);
        let source_tick = ChangeTick::new(11);
        let target_tick = ChangeTick::new(37);
        let mut source = ComponentStorage::default();
        source
            .insert_at_tick(
                component_id,
                StorageType::SparseSet,
                source_entity,
                SparseValue(9),
                source_tick,
            )
            .expect("sparse source row should insert");

        let mut rows = source.extract_entity_rows(source_entity, &[component_id]);
        assert!(!source.contains(component_id, source_entity));
        let row = rows.pop().expect("sparse row should transfer");
        assert_eq!(row.source_ticks(), ComponentTicks::new(source_tick));

        let mut target = ComponentStorage::default();
        target
            .insert_transferred_row(component_id, target_entity, row, target_tick)
            .expect("validated sparse row transfer should succeed");
        assert_eq!(
            target.get::<SparseValue>(component_id, target_entity),
            Some(&SparseValue(9))
        );
        assert_eq!(
            target.ticks(component_id, target_entity),
            Some(ComponentTicks::new(target_tick))
        );
    }

    #[test]
    fn dense_value_insertion_is_rejected_by_the_sparse_owner() {
        let component_id = ComponentId::new(3);
        let error = ComponentStorage::default()
            .insert(
                component_id,
                StorageType::Table,
                InternalEntity::new(1, 0),
                7_u32,
            )
            .expect_err("dense values must be owned by ArchetypeTable");
        assert_eq!(error, StorageError::TableOwnedByArchetype { component_id });
    }
}
