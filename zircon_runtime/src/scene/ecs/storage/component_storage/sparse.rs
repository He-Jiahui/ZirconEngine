use crate::scene::ecs::{ChangeTick, ComponentTicks, InternalEntity};

use super::entry::{RawRemoveResult, StoredComponent};

#[path = "sparse/locator.rs"]
mod locator;

#[cfg(test)]
use locator::SPARSE_LOCATOR_PAGE_SLOTS;
use locator::{SparseRowLocation, SparseRowLocator};

#[derive(Default)]
pub(in crate::scene::ecs::storage) struct SparseComponentStorage {
    entities: Vec<InternalEntity>,
    entries: Vec<SparseEntry>,
    locator: SparseRowLocator,
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
        let location = self.locator.get(entity.index())?;
        (location.generation() == entity.generation()).then(|| location.dense_row())
    }

    fn set_sparse_row(&mut self, entity: InternalEntity, dense_row: usize) {
        self.locator.insert(
            entity.index(),
            SparseRowLocation::new(entity.generation(), dense_row),
        );
    }

    fn remove_sparse_row(&mut self, entity: InternalEntity) -> Option<usize> {
        let location = self.locator.get(entity.index())?;
        if location.generation() != entity.generation() {
            return None;
        }
        self.locator
            .remove(entity.index())
            .map(SparseRowLocation::dense_row)
    }

    #[cfg(test)]
    fn locator_page_count(&self) -> usize {
        self.locator.page_count()
    }

    #[cfg(test)]
    fn locator_slot_capacity(&self) -> usize {
        self.locator.page_count() * SPARSE_LOCATOR_PAGE_SLOTS
    }

    #[cfg(test)]
    fn locator_allocated_bytes(&self) -> usize {
        self.locator.allocated_bytes()
    }

    #[cfg(test)]
    fn locator_flat_prefix_slots(&self) -> usize {
        self.locator.flat_prefix_slots()
    }

    #[cfg(test)]
    fn locator_flat_location_count(&self) -> usize {
        self.locator.flat_location_count()
    }

    #[cfg(test)]
    fn locator_flat_window_base(&self) -> u32 {
        self.locator.flat_window_base()
    }

    #[cfg(test)]
    fn locator_flat_window_slots(&self) -> usize {
        self.locator.flat_window_slots()
    }

    #[cfg(test)]
    fn locator_sparse_page_count(&self) -> usize {
        self.locator.sparse_page_count()
    }

    #[cfg(test)]
    fn locator_sparse_directory_capacity(&self) -> usize {
        self.locator.sparse_directory_capacity()
    }
}

#[cfg(test)]
#[path = "sparse/tests.rs"]
mod tests;
