use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::scene::ecs::{ChangeTick, ComponentTicks, InternalEntity};

use super::entry::{RawRemoveResult, StoredComponent};

#[derive(Default)]
pub(in crate::scene::ecs::storage) struct TableComponentStorage {
    rows: HashMap<InternalEntity, usize>,
    entries: Vec<TableEntry>,
}

struct TableEntry {
    entity: InternalEntity,
    value: StoredComponent,
    ticks: ComponentTicks,
}

impl TableComponentStorage {
    pub(super) fn insert(
        &mut self,
        entity: InternalEntity,
        value: StoredComponent,
        tick: ChangeTick,
    ) -> Option<StoredComponent> {
        match self.rows.entry(entity) {
            Entry::Occupied(entry) => {
                let row = *entry.get();
                self.entries[row].ticks.set_changed(tick);
                Some(std::mem::replace(&mut self.entries[row].value, value))
            }
            Entry::Vacant(entry) => {
                let row = self.entries.len();
                self.entries.push(TableEntry {
                    entity,
                    value,
                    ticks: ComponentTicks::new(tick),
                });
                entry.insert(row);
                None
            }
        }
    }

    pub(super) fn get<T>(&self, entity: InternalEntity) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        let row = *self.rows.get(&entity)?;
        self.entries[row].value.downcast_ref::<T>()
    }

    pub(super) fn get_mut<T>(&mut self, entity: InternalEntity) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        let row = self.rows.get(&entity).copied()?;
        self.entries[row].value.downcast_mut::<T>()
    }

    pub(super) fn get_mut_at_tick<T>(
        &mut self,
        entity: InternalEntity,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        let row = self.rows.get(&entity).copied()?;
        let entry = &mut self.entries[row];
        entry.ticks.set_changed(tick);
        entry.value.downcast_mut::<T>()
    }

    pub(super) fn get_mut_with_ticks<T>(
        &mut self,
        entity: InternalEntity,
    ) -> Option<(&mut T, &mut ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        let row = self.rows.get(&entity).copied()?;
        let TableEntry { value, ticks, .. } = &mut self.entries[row];
        let value = value.downcast_mut::<T>()?;
        Some((value, ticks))
    }

    pub(super) fn remove(&mut self, entity: InternalEntity) -> Option<RawRemoveResult> {
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

    pub(super) fn contains(&self, entity: InternalEntity) -> bool {
        self.rows.contains_key(&entity)
    }

    pub(super) fn row(&self, entity: InternalEntity) -> Option<usize> {
        self.rows.get(&entity).copied()
    }

    pub(super) fn get_row<T>(&self, row: usize) -> Option<(InternalEntity, &T, ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entries.get(row)?;
        let value = entry.value.downcast_ref::<T>()?;
        Some((entry.entity, value, entry.ticks))
    }

    pub(super) fn ticks(&self, entity: InternalEntity) -> Option<ComponentTicks> {
        let row = *self.rows.get(&entity)?;
        Some(self.entries[row].ticks)
    }

    pub(super) fn mark_changed(&mut self, entity: InternalEntity, tick: ChangeTick) {
        let Some(row) = self.rows.get(&entity).copied() else {
            return;
        };
        self.entries[row].ticks.set_changed(tick);
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}
