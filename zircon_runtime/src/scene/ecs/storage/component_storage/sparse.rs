use std::cell::UnsafeCell;

use crate::scene::ecs::{ChangeTick, ComponentTicks, InternalEntity};

use super::entry::{RawRemoveResult, StoredComponent};

#[path = "sparse/locator.rs"]
mod locator;

#[cfg(test)]
use locator::SPARSE_LOCATOR_PAGE_SLOTS;
use locator::{SparseRowLocation, SparseRowLocator};

/// Cold-path structural memory snapshot for one sparse component locator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SparseLocatorDiagnostics {
    pub(super) entry_count: usize,
    pub(super) page_count: usize,
    pub(super) allocated_bytes: usize,
}

#[derive(Default)]
pub(in crate::scene::ecs::storage) struct SparseComponentStorage {
    entities: Vec<InternalEntity>,
    entries: Vec<SparseEntry>,
    locator: SparseRowLocator,
}

// Each payload has its own cell so shared entry access does not freeze
// neighboring values/ticks during a checked unsafe row projection.
struct SparseEntry {
    value: UnsafeCell<StoredComponent>,
    ticks: UnsafeCell<ComponentTicks>,
}

impl SparseEntry {
    /// # Safety
    /// The caller owns this row's value/ticks loan, excluding any conflicting
    /// reference or access to its value even during the concrete type check.
    /// The entry, its Box and its boxed allocation must remain stable.
    unsafe fn typed_value_ptr<T>(&self) -> Option<*mut T>
    where
        T: 'static + Send + Sync,
    {
        // SAFETY: Box's built-in raw place dereference forms no &mut Box and
        // neither moves its ownership nor projects any other entry's payload.
        let value = unsafe { std::ptr::addr_of_mut!(**self.value.get()) };
        // SAFETY: only this selected row's live erased pointee is briefly
        // shared. There may be no prior same-row value loan. The is<T> borrow
        // ends before the caller forms its concrete mutable reference.
        if unsafe { (&*value).is::<T>() } {
            Some(value.cast::<T>())
        } else {
            None
        }
    }
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
            entry.ticks.get_mut().set_changed(tick);
            return Some(std::mem::replace(entry.value.get_mut(), value));
        }
        let row = self.entries.len();
        self.entities.push(entity);
        self.entries.push(SparseEntry {
            value: UnsafeCell::new(value),
            ticks: UnsafeCell::new(ComponentTicks::new(tick)),
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
            *entry.ticks.get_mut() = ticks;
            return Some(std::mem::replace(entry.value.get_mut(), value));
        }
        let row = self.entries.len();
        self.entities.push(entity);
        self.entries.push(SparseEntry {
            value: UnsafeCell::new(value),
            ticks: UnsafeCell::new(ticks),
        });
        self.set_sparse_row(entity, row);
        None
    }

    pub(super) fn get<T>(&self, entity: InternalEntity) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entry(entity)?;
        // SAFETY: safe shared storage access excludes conflicting row loans.
        unsafe { &*entry.value.get() }.downcast_ref::<T>()
    }

    pub(super) fn get_with_ticks<T>(&self, entity: InternalEntity) -> Option<(&T, ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entry(entity)?;
        // SAFETY: safe shared access excludes conflicting value/tick loans.
        let value = unsafe { &*entry.value.get() }.downcast_ref::<T>()?;
        Some((value, unsafe { *entry.ticks.get() }))
    }

    pub(super) fn get_mut<T>(&mut self, entity: InternalEntity) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entry_mut(entity)?;
        entry.value.get_mut().downcast_mut::<T>()
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
        entry.ticks.get_mut().set_changed(tick);
        entry.value.get_mut().downcast_mut::<T>()
    }

    pub(super) fn get_mut_with_ticks<T>(
        &mut self,
        entity: InternalEntity,
    ) -> Option<(&mut T, &mut ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        let SparseEntry { value, ticks } = self.entry_mut(entity)?;
        let value = value.get_mut().downcast_mut::<T>()?;
        Some((value, ticks.get_mut()))
    }

    /// Eagerly mark one live row with the exclusive getter's original tick order.
    ///
    /// # Safety
    /// The caller exclusively owns this entity row's value and ticks for the
    /// returned lifetime, including eager update and type check. No conflicting
    /// reads/writes occur on any thread. Entries, entities, locator and
    /// allocations stay stable; other loans select different rows.
    pub(super) unsafe fn get_mut_at_tick_unchecked<T>(
        &self,
        entity: InternalEntity,
        tick: ChangeTick,
    ) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entry(entity)?;
        // SAFETY: the caller's unique tick loan permits this one-row update.
        // Preserve the exclusive getter: a live wrong-type row is still marked.
        unsafe { &mut *entry.ticks.get() }.set_changed(tick);
        // SAFETY: no same-row value reference exists during this type check.
        let value = unsafe { entry.typed_value_ptr::<T>()? };
        Some(unsafe { &mut *value })
    }

    /// Project only the selected concrete component and its separate tick cell.
    ///
    /// # Safety
    /// The caller exclusively owns this row's value and ticks for both returned
    /// lifetimes, with no conflicting reads/writes on any thread. Entries,
    /// entities, locator and allocations stay stable. All other live value/tick
    /// loans select different rows, including safe shared row references.
    pub(super) unsafe fn get_mut_with_ticks_unchecked<T>(
        &self,
        entity: InternalEntity,
    ) -> Option<(&mut T, &mut ComponentTicks)>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entry(entity)?;
        // SAFETY: the caller owns the checked entity row's value/tick loan.
        let value = unsafe { entry.typed_value_ptr::<T>()? };
        let ticks = entry.ticks.get();
        // SAFETY: both concrete projections are disjoint and uniquely loaned.
        Some(unsafe { (&mut *value, &mut *ticks) })
    }

    /// Check this boxed payload's actual type and copy its ticks.
    /// This briefly shares the selected payload as dyn Any, without returning T.
    /// Unsafe dispatchers must exclude conflicting value and tick loans for it.
    pub(super) fn ticks_for_type<T>(&self, entity: InternalEntity) -> Option<ComponentTicks>
    where
        T: 'static + Send + Sync,
    {
        let entry = self.entry(entity)?;
        // SAFETY: safe shared storage ownership excludes conflicting row loans.
        // Box's raw place dereference does not borrow or move its owning Box.
        let value = unsafe { std::ptr::addr_of!(**entry.value.get()) };
        // SAFETY: only this selected payload is briefly shared for its real type.
        let matches = unsafe { (&*value).is::<T>() };
        if !matches {
            return None;
        }
        // SAFETY: the selected tick cell has no conflicting mutable tick loan.
        Some(unsafe { *entry.ticks.get() })
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
            value: entry.value.into_inner(),
            ticks: entry.ticks.into_inner(),
        })
    }

    pub(super) fn contains(&self, entity: InternalEntity) -> bool {
        self.dense_row(entity).is_some()
    }

    pub(super) fn ticks(&self, entity: InternalEntity) -> Option<ComponentTicks> {
        let entry = self.entry(entity)?;
        // SAFETY: safe shared ownership excludes a conflicting tick loan.
        Some(unsafe { *entry.ticks.get() })
    }

    pub(super) fn mark_changed(&mut self, entity: InternalEntity, tick: ChangeTick) {
        if let Some(entry) = self.entry_mut(entity) {
            entry.ticks.get_mut().set_changed(tick);
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

    pub(super) fn locator_diagnostics(&self) -> SparseLocatorDiagnostics {
        self.locator.diagnostics()
    }

    fn entry(&self, entity: InternalEntity) -> Option<&SparseEntry> {
        let row = self.dense_row(entity)?;
        self.entries.get(row)
    }

    fn entry_mut(&mut self, entity: InternalEntity) -> Option<&mut SparseEntry> {
        let row = self.dense_row(entity)?;
        self.entries.get_mut(row)
    }

    // 槽索引只定位候选记录；代次必须一致，才能把当前位置交给密集数组访问。
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
        self.locator_diagnostics().page_count
    }

    #[cfg(test)]
    fn locator_slot_capacity(&self) -> usize {
        self.locator_diagnostics().page_count * SPARSE_LOCATOR_PAGE_SLOTS
    }

    #[cfg(test)]
    fn locator_allocated_bytes(&self) -> usize {
        self.locator_diagnostics().allocated_bytes
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

// Send follows from the owned Send boxes, tick cells and locator.
// SAFETY: safe shared methods only read Sync components/ticks. Unsafe shared
// writers require a stable, unique value/tick row loan and synchronization
// against all conflicting access; no safe method grants that mutable loan.
unsafe impl Sync for SparseComponentStorage {}

#[cfg(test)]
#[path = "sparse/tests/cases.rs"]
mod tests;
