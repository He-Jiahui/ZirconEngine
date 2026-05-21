use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;

use crate::scene::ecs::{ChangeTick, ComponentId, ComponentTicks, InternalEntity, StorageType};

use super::{ComponentRemoveResult, StorageError};

type StoredComponent = Box<dyn Any + Send + Sync>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComponentStorageLocation {
    pub component_id: ComponentId,
    pub storage_type: StorageType,
    pub entity: InternalEntity,
    pub table_row: Option<usize>,
}

#[derive(Default)]
pub struct ComponentStorage {
    storage_types: HashMap<ComponentId, StorageType>,
    component_types: HashMap<ComponentId, TypeId>,
    table_components: HashMap<ComponentId, TableComponentStorage>,
    sparse_components: HashMap<ComponentId, SparseComponentStorage>,
}

#[derive(Default)]
struct TableComponentStorage {
    rows: HashMap<InternalEntity, usize>,
    entries: Vec<TableEntry>,
}

struct TableEntry {
    entity: InternalEntity,
    value: StoredComponent,
    ticks: ComponentTicks,
}

#[derive(Default)]
struct SparseComponentStorage {
    entries: HashMap<InternalEntity, SparseEntry>,
}

struct SparseEntry {
    value: StoredComponent,
    ticks: ComponentTicks,
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
        old.map(|old| downcast_component(component_id, old))
            .transpose()
    }

    pub fn get<T>(&self, component_id: ComponentId, entity: InternalEntity) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        match self.storage_types.get(&component_id).copied()? {
            StorageType::Table => self
                .table_components
                .get(&component_id)
                .and_then(|storage| storage.get(entity)),
            StorageType::SparseSet => self
                .sparse_components
                .get(&component_id)
                .and_then(|storage| storage.get(entity)),
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
            StorageType::Table => self
                .table_components
                .get_mut(&component_id)
                .and_then(|storage| storage.get_mut(entity)),
            StorageType::SparseSet => self
                .sparse_components
                .get_mut(&component_id)
                .and_then(|storage| storage.get_mut(entity)),
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
        self.mark_changed(component_id, entity, tick);
        self.get_mut(component_id, entity)
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
            StorageType::Table => self
                .table_components
                .get_mut(&component_id)
                .and_then(|storage| storage.remove(entity)),
            StorageType::SparseSet => self
                .sparse_components
                .get_mut(&component_id)
                .and_then(|storage| storage.remove(entity)),
        };
        removed
            .map(|removed| {
                Ok(ComponentRemoveResult {
                    value: downcast_component(component_id, removed.value)?,
                    swapped_entity: removed.swapped_entity,
                })
            })
            .transpose()
    }

    pub fn contains(&self, component_id: ComponentId, entity: InternalEntity) -> bool {
        match self.storage_types.get(&component_id).copied() {
            Some(StorageType::Table) => self
                .table_components
                .get(&component_id)
                .is_some_and(|storage| storage.contains(entity)),
            Some(StorageType::SparseSet) => self
                .sparse_components
                .get(&component_id)
                .is_some_and(|storage| storage.contains(entity)),
            None => false,
        }
    }

    pub fn ticks(
        &self,
        component_id: ComponentId,
        entity: InternalEntity,
    ) -> Option<ComponentTicks> {
        match self.storage_types.get(&component_id).copied()? {
            StorageType::Table => self
                .table_components
                .get(&component_id)
                .and_then(|storage| storage.ticks(entity)),
            StorageType::SparseSet => self
                .sparse_components
                .get(&component_id)
                .and_then(|storage| storage.ticks(entity)),
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
            StorageType::SparseSet => self
                .sparse_components
                .get(&component_id)?
                .contains(entity)
                .then_some(ComponentStorageLocation {
                    component_id,
                    storage_type: StorageType::SparseSet,
                    entity,
                    table_row: None,
                }),
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
        self.table_components
            .get(&component_id)
            .and_then(|storage| storage.get_row(row))
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
                (entity == location.entity).then_some((value, ticks))
            }
            StorageType::SparseSet => {
                if location.table_row.is_some() {
                    return None;
                }
                let value = self.get::<T>(location.component_id, location.entity)?;
                let ticks = self.ticks(location.component_id, location.entity)?;
                Some((value, ticks))
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
        let mut removed = Vec::new();
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
        removed.sort_unstable();
        removed
    }

    pub(crate) fn component_ids_for_entity(&self, entity: InternalEntity) -> Vec<ComponentId> {
        let mut component_ids = Vec::new();
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
        component_ids.sort_unstable();
        component_ids
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
        if let Some(existing) = self.storage_types.get(&component_id).copied() {
            if existing != requested {
                return Err(StorageError::StorageTypeMismatch {
                    component_id,
                    existing,
                    requested,
                });
            }
        } else {
            self.storage_types.insert(component_id, requested);
        }
        Ok(())
    }

    fn ensure_component_type<T>(&mut self, component_id: ComponentId) -> Result<(), StorageError>
    where
        T: 'static + Send + Sync,
    {
        let requested = TypeId::of::<T>();
        if let Some(existing) = self.component_types.get(&component_id).copied() {
            if existing != requested {
                return Err(StorageError::ComponentTypeMismatch { component_id });
            }
        } else {
            self.component_types.insert(component_id, requested);
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

impl TableComponentStorage {
    fn insert(
        &mut self,
        entity: InternalEntity,
        value: StoredComponent,
        tick: ChangeTick,
    ) -> Option<StoredComponent> {
        if let Some(row) = self.rows.get(&entity).copied() {
            self.entries[row].ticks.set_changed(tick);
            return Some(std::mem::replace(&mut self.entries[row].value, value));
        }
        let row = self.entries.len();
        self.entries.push(TableEntry {
            entity,
            value,
            ticks: ComponentTicks::new(tick),
        });
        self.rows.insert(entity, row);
        None
    }

    fn get<T>(&self, entity: InternalEntity) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        self.rows
            .get(&entity)
            .and_then(|row| self.entries.get(*row))
            .and_then(|entry| entry.value.downcast_ref::<T>())
    }

    fn get_mut<T>(&mut self, entity: InternalEntity) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        self.rows
            .get(&entity)
            .copied()
            .and_then(|row| self.entries.get_mut(row))
            .and_then(|entry| entry.value.downcast_mut::<T>())
    }

    fn remove(&mut self, entity: InternalEntity) -> Option<RawRemoveResult> {
        let row = self.rows.remove(&entity)?;
        let last_row = self.entries.len() - 1;
        let removed = self.entries.swap_remove(row);
        let swapped_entity = if row != last_row {
            let moved_entity = self.entries[row].entity;
            self.rows.insert(moved_entity, row);
            Some(moved_entity)
        } else {
            None
        };
        Some(RawRemoveResult {
            value: removed.value,
            swapped_entity,
        })
    }

    fn contains(&self, entity: InternalEntity) -> bool {
        self.rows.contains_key(&entity)
    }

    fn row(&self, entity: InternalEntity) -> Option<usize> {
        self.rows.get(&entity).copied()
    }

    fn get_row<T>(&self, row: usize) -> Option<(InternalEntity, &T, ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entries.get(row)?;
        let value = entry.value.downcast_ref::<T>()?;
        Some((entry.entity, value, entry.ticks))
    }

    fn ticks(&self, entity: InternalEntity) -> Option<ComponentTicks> {
        self.rows
            .get(&entity)
            .and_then(|row| self.entries.get(*row))
            .map(|entry| entry.ticks)
    }

    fn mark_changed(&mut self, entity: InternalEntity, tick: ChangeTick) {
        if let Some(row) = self.rows.get(&entity).copied() {
            if let Some(entry) = self.entries.get_mut(row) {
                entry.ticks.set_changed(tick);
            }
        }
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl SparseComponentStorage {
    fn insert(
        &mut self,
        entity: InternalEntity,
        value: StoredComponent,
        tick: ChangeTick,
    ) -> Option<StoredComponent> {
        match self.entries.insert(
            entity,
            SparseEntry {
                value,
                ticks: ComponentTicks::new(tick),
            },
        ) {
            Some(old) => {
                if let Some(entry) = self.entries.get_mut(&entity) {
                    entry.ticks = old.ticks;
                    entry.ticks.set_changed(tick);
                }
                Some(old.value)
            }
            None => None,
        }
    }

    fn get<T>(&self, entity: InternalEntity) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        self.entries
            .get(&entity)
            .and_then(|entry| entry.value.downcast_ref::<T>())
    }

    fn get_mut<T>(&mut self, entity: InternalEntity) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        self.entries
            .get_mut(&entity)
            .and_then(|entry| entry.value.downcast_mut::<T>())
    }

    fn remove(&mut self, entity: InternalEntity) -> Option<RawRemoveResult> {
        self.entries.remove(&entity).map(|entry| RawRemoveResult {
            value: entry.value,
            swapped_entity: None,
        })
    }

    fn contains(&self, entity: InternalEntity) -> bool {
        self.entries.contains_key(&entity)
    }

    fn ticks(&self, entity: InternalEntity) -> Option<ComponentTicks> {
        self.entries.get(&entity).map(|entry| entry.ticks)
    }

    fn mark_changed(&mut self, entity: InternalEntity, tick: ChangeTick) {
        if let Some(entry) = self.entries.get_mut(&entity) {
            entry.ticks.set_changed(tick);
        }
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

struct RawRemoveResult {
    value: StoredComponent,
    swapped_entity: Option<InternalEntity>,
}

fn downcast_component<T>(
    component_id: ComponentId,
    value: StoredComponent,
) -> Result<T, StorageError>
where
    T: 'static + Send + Sync,
{
    value
        .downcast::<T>()
        .map(|value| *value)
        .map_err(|_| StorageError::ComponentTypeMismatch { component_id })
}
