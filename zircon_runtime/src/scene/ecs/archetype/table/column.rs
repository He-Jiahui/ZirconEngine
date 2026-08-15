use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::ptr::NonNull;

use crate::scene::ecs::component::TableColumnLayout;
use crate::scene::ecs::{storage::StoredComponent, ChangeTick, ComponentTicks};

/// Owns the contiguous body and tick rows for exactly one registered table component.
pub(super) struct ArchetypeColumn {
    layout: TableColumnLayout,
    data: NonNull<u8>,
    capacity: usize,
    ticks: Vec<ComponentTicks>,
}

impl ArchetypeColumn {
    pub(super) fn new(layout: TableColumnLayout) -> Self {
        Self {
            layout,
            data: NonNull::dangling(),
            capacity: 0,
            ticks: Vec::new(),
        }
    }

    pub(super) fn accepts(&self, value: &StoredComponent) -> bool {
        self.layout.accepts(value)
    }

    pub(super) fn type_name(&self) -> &'static str {
        self.layout.type_name()
    }

    pub(super) fn estimated_heap_bytes(&self) -> usize {
        self.capacity
            .saturating_mul(self.layout.layout().size())
            .saturating_add(
                self.ticks
                    .capacity()
                    .saturating_mul(std::mem::size_of::<ComponentTicks>()),
            )
    }

    pub(super) fn push(&mut self, value: StoredComponent, ticks: ComponentTicks) {
        debug_assert!(self.accepts(&value));
        let row = self.ticks.len();
        self.reserve(row.saturating_add(1));
        let destination = self.slot_ptr(row);
        // SAFETY: `reserve` created an aligned uninitialized slot for this
        // registered component, and the caller prevalidated its concrete type.
        unsafe { self.layout.write_box(value, destination) };
        self.ticks.push(ticks);
    }

    pub(super) fn get<T>(&self, row: usize) -> Option<&T>
    where
        T: Send + Sync + 'static,
    {
        let value = self.typed_slot_ptr::<T>(row)?;
        // SAFETY: `typed_slot_ptr` validates the registered TypeId and row.
        Some(unsafe { &*value })
    }

    pub(super) fn get_mut<T>(&mut self, row: usize) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        let value = self.typed_slot_ptr::<T>(row)?;
        // SAFETY: `typed_slot_ptr` validates the registered TypeId and row;
        // this method has exclusive access to the column.
        Some(unsafe { &mut *value })
    }

    pub(super) fn get_mut_at_tick<T>(&mut self, row: usize, tick: ChangeTick) -> Option<&mut T>
    where
        T: Send + Sync + 'static,
    {
        let value = self.typed_slot_ptr::<T>(row)?;
        self.ticks.get_mut(row)?.set_changed(tick);
        // SAFETY: `typed_slot_ptr` validates the registered TypeId and row;
        // the tick update is disjoint from the component body allocation.
        Some(unsafe { &mut *value })
    }

    pub(super) fn get_mut_with_ticks<T>(
        &mut self,
        row: usize,
    ) -> Option<(&mut T, &mut ComponentTicks)>
    where
        T: Send + Sync + 'static,
    {
        let value = self.typed_slot_ptr::<T>(row)?;
        let ticks = self.ticks.get_mut(row)?;
        // SAFETY: `typed_slot_ptr` validates the registered TypeId and row.
        // The component allocation and tick vector are disjoint allocations.
        Some((unsafe { &mut *value }, ticks))
    }

    pub(super) fn ticks(&self, row: usize) -> Option<ComponentTicks> {
        self.ticks.get(row).copied()
    }

    pub(super) fn replace(
        &mut self,
        row: usize,
        value: StoredComponent,
        tick: ChangeTick,
    ) -> Option<StoredComponent> {
        if !self.accepts(&value) || row >= self.ticks.len() {
            return None;
        }
        let slot = self.slot_ptr(row);
        // SAFETY: the row is initialized and its layout has already accepted
        // the replacement value. `take_box` moves the old body out before the
        // same slot is initialized with the new body.
        let previous = unsafe { self.layout.take_box(slot) };
        // SAFETY: validation above proves `value` matches this slot layout.
        unsafe { self.layout.write_box(value, slot) };
        self.ticks[row].set_changed(tick);
        Some(previous)
    }

    pub(super) fn take(&mut self, row: usize) -> Option<(StoredComponent, ComponentTicks)> {
        let len = self.ticks.len();
        if row >= len {
            return None;
        }
        let last_row = len - 1;
        let slot = self.slot_ptr(row);
        // SAFETY: the row is initialized and exclusively owned by this column.
        let value = unsafe { self.layout.take_box(slot) };
        if !self.layout.is_zero_sized() && row != last_row {
            let last = self.slot_ptr(last_row);
            // SAFETY: source and destination are distinct initialized slots of
            // equal layout. The last source becomes outside the logical length.
            unsafe {
                std::ptr::copy_nonoverlapping(last, slot, self.layout.layout().size());
            }
        }
        let ticks = self.ticks.swap_remove(row);
        Some((value, ticks))
    }

    fn reserve(&mut self, required: usize) {
        if required <= self.capacity {
            return;
        }
        let grown = self.capacity.saturating_add((self.capacity / 2).max(1));
        let new_capacity = grown.max(required);
        self.ticks.reserve(new_capacity - self.ticks.len());
        if !self.layout.is_zero_sized() {
            let new_layout = self.allocation_layout(new_capacity);
            let data = if self.capacity == 0 {
                // SAFETY: `new_layout` is a valid non-zero allocation layout.
                unsafe { alloc(new_layout) }
            } else {
                let old_layout = self.allocation_layout(self.capacity);
                // SAFETY: `data` was allocated with `old_layout` and remains
                // exclusively owned by this column.
                unsafe { realloc(self.data.as_ptr(), old_layout, new_layout.size()) }
            };
            let Some(data) = NonNull::new(data) else {
                handle_alloc_error(new_layout);
            };
            self.data = data;
        }
        self.capacity = new_capacity;
    }

    fn allocation_layout(&self, capacity: usize) -> Layout {
        let size = self
            .layout
            .layout()
            .size()
            .checked_mul(capacity)
            .expect("dense archetype column allocation size overflow");
        Layout::from_size_align(size, self.layout.layout().align())
            .expect("dense archetype column allocation layout must remain valid")
    }

    fn typed_slot_ptr<T>(&self, row: usize) -> Option<*mut T>
    where
        T: Send + Sync + 'static,
    {
        if !self.layout.matches::<T>() || row >= self.ticks.len() {
            return None;
        }
        if self.layout.is_zero_sized() {
            return Some(NonNull::<T>::dangling().as_ptr());
        }
        Some(self.slot_ptr(row).cast::<T>())
    }

    fn slot_ptr(&self, row: usize) -> *mut u8 {
        if self.layout.is_zero_sized() {
            return self.data.as_ptr();
        }
        debug_assert!(row < self.capacity);
        // SAFETY: callers only request allocated rows and multiplication is
        // bounded by the allocation layout created in `reserve`.
        unsafe { self.data.as_ptr().add(row * self.layout.layout().size()) }
    }
}

impl Drop for ArchetypeColumn {
    fn drop(&mut self) {
        for row in 0..self.ticks.len() {
            let slot = self.slot_ptr(row);
            // SAFETY: every row below `ticks.len()` owns exactly one live body.
            unsafe { self.layout.drop_value(slot) };
        }
        if !self.layout.is_zero_sized() && self.capacity != 0 {
            let layout = self.allocation_layout(self.capacity);
            // SAFETY: `data` was allocated by `reserve` with this exact layout.
            unsafe { dealloc(self.data.as_ptr(), layout) };
        }
    }
}

// SAFETY: construction is limited to `T: Send + Sync` component layouts and
// all raw body access is gated by exclusive table ownership.
unsafe impl Send for ArchetypeColumn {}
// SAFETY: immutable access returns typed shared references only after checking
// the registered layout; no interior mutability is exposed by the column.
unsafe impl Sync for ArchetypeColumn {}
