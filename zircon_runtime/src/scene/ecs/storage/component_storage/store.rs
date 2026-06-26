use std::any::TypeId;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;

use crate::scene::ecs::{ChangeTick, ComponentId, ComponentTicks, InternalEntity, StorageType};

use super::super::{ComponentRemoveResult, StorageError};
use super::component_results::{downcast_component, sort_component_ids_if_needed};
use super::location::ComponentStorageLocation;
use super::sparse::SparseComponentStorage;
use super::table::TableComponentStorage;

#[derive(Default)]
pub struct ComponentStorage {
    storage_types: HashMap<ComponentId, StorageType>,
    component_types: HashMap<ComponentId, TypeId>,
    table_components: HashMap<ComponentId, TableComponentStorage>,
    sparse_components: HashMap<ComponentId, SparseComponentStorage>,
}

impl ComponentStorage {
    pub fn insert<T>(
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

    pub fn insert_at_tick<T>(
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
        self.ensure_storage_type(component_id, storage_type)?;
        self.ensure_component_type::<T>(component_id)?;
        let old = match storage_type {
            StorageType::Table => self
                .table_components
                .entry(component_id)
                .or_default()
                .insert(entity, Box::new(value), tick),
            StorageType::SparseSet => self
                .sparse_components
                .entry(component_id)
                .or_default()
                .insert(entity, Box::new(value), tick),
        };
        let Some(old) = old else {
            return Ok(None);
        };
        Ok(Some(downcast_component(component_id, old)?))
    }

    pub fn get<T>(&self, component_id: ComponentId, entity: InternalEntity) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        match self.storage_types.get(&component_id).copied()? {
            StorageType::Table => {
                let storage = self.table_components.get(&component_id)?;
                storage.get(entity)
            }
            StorageType::SparseSet => {
                let storage = self.sparse_components.get(&component_id)?;
                storage.get(entity)
            }
        }
    }

    pub fn get_mut<T>(
        &mut self,
        component_id: ComponentId,
        entity: InternalEntity,
    ) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        match self.storage_types.get(&component_id).copied()? {
            StorageType::Table => {
                let storage = self.table_components.get_mut(&component_id)?;
                storage.get_mut(entity)
            }
            StorageType::SparseSet => {
                let storage = self.sparse_components.get_mut(&component_id)?;
                storage.get_mut(entity)
            }
        }
    }

    pub fn get_mut_at_tick<T>(
        &mut self,
        component_id: ComponentId,
        entity: InternalEntity,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        match self.storage_types.get(&component_id).copied()? {
            StorageType::Table => {
                let storage = self.table_components.get_mut(&component_id)?;
                storage.get_mut_at_tick(entity, tick)
            }
            StorageType::SparseSet => {
                let storage = self.sparse_components.get_mut(&component_id)?;
                storage.get_mut_at_tick(entity, tick)
            }
        }
    }

    pub fn remove<T>(
        &mut self,
        component_id: ComponentId,
        entity: InternalEntity,
    ) -> Result<Option<ComponentRemoveResult<T>>, StorageError>
    where
        T: 'static + Send + Sync,
    {
        let Some(storage_type) = self.storage_types.get(&component_id).copied() else {
            return Ok(None);
        };
        self.ensure_component_type::<T>(component_id)?;
        let removed = match storage_type {
            StorageType::Table => {
                let Some(storage) = self.table_components.get_mut(&component_id) else {
                    return Ok(None);
                };
                storage.remove(entity)
            }
            StorageType::SparseSet => {
                let Some(storage) = self.sparse_components.get_mut(&component_id) else {
                    return Ok(None);
                };
                storage.remove(entity)
            }
        };
        let Some(removed) = removed else {
            return Ok(None);
        };
        Ok(Some(ComponentRemoveResult {
            value: downcast_component(component_id, removed.value)?,
            swapped_entity: removed.swapped_entity,
        }))
    }

    pub fn contains(&self, component_id: ComponentId, entity: InternalEntity) -> bool {
        match self.storage_types.get(&component_id).copied() {
            Some(StorageType::Table) => {
                let Some(storage) = self.table_components.get(&component_id) else {
                    return false;
                };
                storage.contains(entity)
            }
            Some(StorageType::SparseSet) => {
                let Some(storage) = self.sparse_components.get(&component_id) else {
                    return false;
                };
                storage.contains(entity)
            }
            None => false,
        }
    }

    pub fn ticks(
        &self,
        component_id: ComponentId,
        entity: InternalEntity,
    ) -> Option<ComponentTicks> {
        match self.storage_types.get(&component_id).copied()? {
            StorageType::Table => {
                let storage = self.table_components.get(&component_id)?;
                storage.ticks(entity)
            }
            StorageType::SparseSet => {
                let storage = self.sparse_components.get(&component_id)?;
                storage.ticks(entity)
            }
        }
    }

    pub fn location(
        &self,
        component_id: ComponentId,
        entity: InternalEntity,
    ) -> Option<ComponentStorageLocation> {
        match self.storage_types.get(&component_id).copied()? {
            StorageType::Table => {
                let row = self.table_components.get(&component_id)?.row(entity)?;
                Some(ComponentStorageLocation {
                    component_id,
                    storage_type: StorageType::Table,
                    entity,
                    table_row: Some(row),
                })
            }
            StorageType::SparseSet => {
                let storage = self.sparse_components.get(&component_id)?;
                if !storage.contains(entity) {
                    return None;
                }
                Some(ComponentStorageLocation {
                    component_id,
                    storage_type: StorageType::SparseSet,
                    entity,
                    table_row: None,
                })
            }
        }
    }

    pub fn get_table_row<T>(
        &self,
        component_id: ComponentId,
        row: usize,
    ) -> Option<(InternalEntity, &T, ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        if self.storage_types.get(&component_id).copied()? != StorageType::Table {
            return None;
        }
        let storage = self.table_components.get(&component_id)?;
        storage.get_row(row)
    }

    pub fn get_with_ticks_at_location<T>(
        &self,
        location: ComponentStorageLocation,
    ) -> Option<(&T, ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        match location.storage_type {
            StorageType::Table => {
                let row = location.table_row?;
                let (entity, value, ticks) = self.get_table_row::<T>(location.component_id, row)?;
                if entity != location.entity {
                    return None;
                }
                Some((value, ticks))
            }
            StorageType::SparseSet => {
                if location.table_row.is_some() {
                    return None;
                }
                self.sparse_components
                    .get(&location.component_id)?
                    .get_with_ticks(location.entity)
            }
        }
    }

    pub fn mark_changed(
        &mut self,
        component_id: ComponentId,
        entity: InternalEntity,
        tick: ChangeTick,
    ) {
        match self.storage_types.get(&component_id).copied() {
            Some(StorageType::Table) => {
                if let Some(storage) = self.table_components.get_mut(&component_id) {
                    storage.mark_changed(entity, tick);
                }
            }
            Some(StorageType::SparseSet) => {
                if let Some(storage) = self.sparse_components.get_mut(&component_id) {
                    storage.mark_changed(entity, tick);
                }
            }
            None => {}
        }
    }

    pub fn remove_entity(&mut self, entity: InternalEntity) -> Vec<ComponentId> {
        let mut removed = Vec::with_capacity(self.component_storage_count());
        for (component_id, storage) in self.table_components.iter_mut() {
            if storage.remove(entity).is_some() {
                removed.push(*component_id);
            }
        }
        for (component_id, storage) in self.sparse_components.iter_mut() {
            if storage.remove(entity).is_some() {
                removed.push(*component_id);
            }
        }
        sort_component_ids_if_needed(&mut removed);
        removed
    }

    pub(crate) fn component_ids_for_entity(&self, entity: InternalEntity) -> Vec<ComponentId> {
        let mut component_ids = Vec::with_capacity(self.component_storage_count());
        for (component_id, storage) in &self.table_components {
            if storage.contains(entity) {
                component_ids.push(*component_id);
            }
        }
        for (component_id, storage) in &self.sparse_components {
            if storage.contains(entity) {
                component_ids.push(*component_id);
            }
        }
        sort_component_ids_if_needed(&mut component_ids);
        component_ids
    }

    pub(crate) fn component_ids_for_entity_by_storage(
        &self,
        entity: InternalEntity,
        table_components: &mut Vec<ComponentId>,
        sparse_set_components: &mut Vec<ComponentId>,
    ) {
        table_components.clear();
        table_components.reserve(self.table_components.len());
        for (component_id, storage) in &self.table_components {
            if storage.contains(entity) {
                table_components.push(*component_id);
            }
        }

        sparse_set_components.clear();
        sparse_set_components.reserve(self.sparse_components.len());
        for (component_id, storage) in &self.sparse_components {
            if storage.contains(entity) {
                sparse_set_components.push(*component_id);
            }
        }
    }

    fn component_storage_count(&self) -> usize {
        self.table_components.len() + self.sparse_components.len()
    }

    pub fn storage_type(&self, component_id: ComponentId) -> Option<StorageType> {
        self.storage_types.get(&component_id).copied()
    }

    pub fn len_for_component(&self, component_id: ComponentId) -> usize {
        match self.storage_types.get(&component_id).copied() {
            Some(StorageType::Table) => self
                .table_components
                .get(&component_id)
                .map_or(0, TableComponentStorage::len),
            Some(StorageType::SparseSet) => self
                .sparse_components
                .get(&component_id)
                .map_or(0, SparseComponentStorage::len),
            None => 0,
        }
    }

    fn ensure_storage_type(
        &mut self,
        component_id: ComponentId,
        requested: StorageType,
    ) -> Result<(), StorageError> {
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
        let mut storage_types = Vec::with_capacity(self.storage_types.len());
        for entry in &self.storage_types {
            storage_types.push(entry);
        }
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
