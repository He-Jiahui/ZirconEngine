use crate::scene::ecs::{ChangeTick, ComponentTicks, InternalEntity};

use super::entry::{RawRemoveResult, StoredComponent};

#[derive(Default)]
pub(in crate::scene::ecs::storage) struct SparseComponentStorage {
    entities: Vec<InternalEntity>,
    entries: Vec<SparseEntry>,
    sparse_rows: Vec<Option<SparseRowLocation>>,
}

#[derive(Clone, Copy)]
struct SparseRowLocation {
    generation: u32,
    dense_row: usize,
}

struct SparseEntry {
    value: StoredComponent,
    ticks: ComponentTicks,
}

impl SparseComponentStorage {
    pub(super) fn insert(
        &mut self,
        entity: InternalEntity,
        value: StoredComponent,
        tick: ChangeTick,
    ) -> Option<StoredComponent> {
        if let Some(row) = self.dense_row(entity) {
            let entry = &mut self.entries[row];
            entry.ticks.set_changed(tick);
            return Some(std::mem::replace(&mut entry.value, value));
        }
        let row = self.entries.len();
        self.entities.push(entity);
        self.entries.push(SparseEntry {
            value,
            ticks: ComponentTicks::new(tick),
        });
        self.set_sparse_row(entity, row);
        None
    }

    pub(super) fn insert_with_ticks(
        &mut self,
        entity: InternalEntity,
        value: StoredComponent,
        ticks: ComponentTicks,
    ) -> Option<StoredComponent> {
        if let Some(row) = self.dense_row(entity) {
            let entry = &mut self.entries[row];
            entry.ticks = ticks;
            return Some(std::mem::replace(&mut entry.value, value));
        }
        let row = self.entries.len();
        self.entities.push(entity);
        self.entries.push(SparseEntry { value, ticks });
        self.set_sparse_row(entity, row);
        None
    }

    pub(super) fn get<T>(&self, entity: InternalEntity) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entry(entity)?;
        entry.value.downcast_ref::<T>()
    }

    pub(super) fn get_with_ticks<T>(&self, entity: InternalEntity) -> Option<(&T, ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entry(entity)?;
        let value = entry.value.downcast_ref::<T>()?;
        Some((value, entry.ticks))
    }

    pub(super) fn get_mut<T>(&mut self, entity: InternalEntity) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entry_mut(entity)?;
        entry.value.downcast_mut::<T>()
    }

    pub(super) fn get_mut_at_tick<T>(
        &mut self,
        entity: InternalEntity,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entry_mut(entity)?;
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
        let SparseEntry { value, ticks } = self.entry_mut(entity)?;
        let value = value.downcast_mut::<T>()?;
        Some((value, ticks))
    }

    pub(super) fn remove(&mut self, entity: InternalEntity) -> Option<RawRemoveResult> {
        let row = self.remove_sparse_row(entity)?;
        let last_row = self.entries.len() - 1;
        let entry = self.entries.swap_remove(row);
        let removed_entity = self.entities.swap_remove(row);
        debug_assert_eq!(removed_entity, entity);
        if row != last_row {
            let swapped_entity = self.entities[row];
            self.set_sparse_row(swapped_entity, row);
        }
        Some(RawRemoveResult {
            value: entry.value,
            ticks: entry.ticks,
        })
    }

    pub(super) fn contains(&self, entity: InternalEntity) -> bool {
        self.dense_row(entity).is_some()
    }

    pub(super) fn ticks(&self, entity: InternalEntity) -> Option<ComponentTicks> {
        let entry = self.entry(entity)?;
        Some(entry.ticks)
    }

    pub(super) fn mark_changed(&mut self, entity: InternalEntity, tick: ChangeTick) {
        if let Some(entry) = self.entry_mut(entity) {
            entry.ticks.set_changed(tick);
        }
    }

    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(super) fn for_each_entity(&self, mut visit: impl FnMut(InternalEntity)) {
        for entity in self.entities.iter().copied() {
            visit(entity);
        }
    }

    fn entry(&self, entity: InternalEntity) -> Option<&SparseEntry> {
        let row = self.dense_row(entity)?;
        self.entries.get(row)
    }

    fn entry_mut(&mut self, entity: InternalEntity) -> Option<&mut SparseEntry> {
        let row = self.dense_row(entity)?;
        self.entries.get_mut(row)
    }

    fn dense_row(&self, entity: InternalEntity) -> Option<usize> {
        let location = self.sparse_rows.get(entity.index() as usize)?.as_ref()?;
        (location.generation == entity.generation()).then_some(location.dense_row)
    }

    fn set_sparse_row(&mut self, entity: InternalEntity, dense_row: usize) {
        let index = entity.index() as usize;
        if self.sparse_rows.len() <= index {
            self.sparse_rows.resize(index + 1, None);
        }
        self.sparse_rows[index] = Some(SparseRowLocation {
            generation: entity.generation(),
            dense_row,
        });
    }

    fn remove_sparse_row(&mut self, entity: InternalEntity) -> Option<usize> {
        let index = entity.index() as usize;
        let location = self.sparse_rows.get(index)?.as_ref()?;
        if location.generation != entity.generation() {
            return None;
        }
        self.sparse_rows[index]
            .take()
            .map(|location| location.dense_row)
    }
}
