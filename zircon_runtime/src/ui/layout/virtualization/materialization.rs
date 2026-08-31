use zircon_runtime_interface::ui::layout::UiVirtualListWindow;

/// A changed physical row slot and its previous/next logical assignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiVirtualListSlotChange {
    pub slot_index: usize,
    pub previous_logical_index: Option<usize>,
    pub logical_index: Option<usize>,
}

/// Retains a bounded physical-slot mapping for a logical fixed-extent list window.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiVirtualListSlotMap {
    slot_logical_indices: Vec<Option<usize>>,
    logical_count: usize,
    window: UiVirtualListWindow,
    generation: u64,
}

impl UiVirtualListSlotMap {
    /// Reconciles assignments without allocating a change list or scanning logical items.
    pub fn reconcile(
        &mut self,
        logical_count: usize,
        slot_capacity: usize,
        requested_window: UiVirtualListWindow,
        changes: &mut Vec<UiVirtualListSlotChange>,
    ) {
        changes.clear();
        let slot_count = slot_capacity.min(logical_count);
        let window = backfill_window_to_slot_count(requested_window, logical_count, slot_count);
        let previous_slot_count = self.slot_logical_indices.len();

        for slot_index in 0..previous_slot_count.max(slot_count) {
            let previous_logical_index =
                self.slot_logical_indices.get(slot_index).copied().flatten();
            let logical_index = (slot_index < slot_count)
                .then(|| logical_index_for_slot(slot_index, slot_count, window))
                .flatten();
            if previous_logical_index != logical_index {
                changes.push(UiVirtualListSlotChange {
                    slot_index,
                    previous_logical_index,
                    logical_index,
                });
            }
        }

        let state_changed = self.logical_count != logical_count
            || self.window != window
            || previous_slot_count != slot_count
            || !changes.is_empty();
        self.slot_logical_indices.resize(slot_count, None);
        for slot_index in 0..slot_count {
            self.slot_logical_indices[slot_index] =
                logical_index_for_slot(slot_index, slot_count, window);
        }
        self.logical_count = logical_count;
        self.window = window;
        if state_changed {
            self.generation = self.generation.wrapping_add(1);
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn logical_count(&self) -> usize {
        self.logical_count
    }

    pub fn window(&self) -> UiVirtualListWindow {
        self.window
    }

    pub fn slot_count(&self) -> usize {
        self.slot_logical_indices.len()
    }

    pub fn active_slot_count(&self) -> usize {
        self.slot_logical_indices
            .iter()
            .filter(|logical_index| logical_index.is_some())
            .count()
    }

    pub fn logical_index_for_slot(&self, slot_index: usize) -> Option<usize> {
        self.slot_logical_indices.get(slot_index).copied().flatten()
    }
}

/// Returns the maximum live row-slot count for a fixed-extent viewport.
pub fn fixed_extent_slot_capacity(
    viewport_extent: f32,
    item_extent: f32,
    overscan: usize,
    logical_count: usize,
) -> usize {
    if logical_count == 0
        || !viewport_extent.is_finite()
        || !item_extent.is_finite()
        || viewport_extent <= 0.0
        || item_extent <= 0.0
    {
        return 0;
    }

    let visible_count = (viewport_extent / item_extent).ceil() as usize;
    visible_count
        .saturating_add(1)
        .saturating_add(overscan.saturating_mul(2))
        .min(logical_count)
}

fn backfill_window_to_slot_count(
    requested: UiVirtualListWindow,
    logical_count: usize,
    slot_count: usize,
) -> UiVirtualListWindow {
    let mut first_visible = requested.first_visible.min(logical_count);
    let mut last_visible_exclusive = requested
        .last_visible_exclusive
        .max(first_visible)
        .min(logical_count)
        .min(first_visible.saturating_add(slot_count));
    let missing = slot_count.saturating_sub(last_visible_exclusive - first_visible);
    let extend_after = missing.min(logical_count - last_visible_exclusive);
    last_visible_exclusive += extend_after;
    first_visible = first_visible.saturating_sub(missing - extend_after);
    UiVirtualListWindow {
        first_visible,
        last_visible_exclusive,
    }
}

fn logical_index_for_slot(
    slot_index: usize,
    slot_count: usize,
    window: UiVirtualListWindow,
) -> Option<usize> {
    if slot_count == 0 || window.first_visible >= window.last_visible_exclusive {
        return None;
    }

    let first_slot = window.first_visible % slot_count;
    let slot_delta = if slot_index >= first_slot {
        slot_index - first_slot
    } else {
        slot_count - (first_slot - slot_index)
    };
    let logical_index = window.first_visible.saturating_add(slot_delta);
    (logical_index < window.last_visible_exclusive).then_some(logical_index)
}

#[cfg(test)]
mod tests {
    use super::{fixed_extent_slot_capacity, UiVirtualListSlotMap};
    use zircon_runtime_interface::ui::layout::UiVirtualListWindow;

    #[test]
    fn slot_count_is_independent_of_logical_count() {
        let capacities = [1, 100, 10_000, 100_000]
            .map(|logical_count| fixed_extent_slot_capacity(80.0, 20.0, 1, logical_count));

        assert_eq!(capacities, [1, 7, 7, 7]);
    }

    #[test]
    fn fractional_scroll_capacity_keeps_both_partial_boundary_items() {
        let capacity = fixed_extent_slot_capacity(80.0, 20.0, 0, 100);
        let mut slots = UiVirtualListSlotMap::default();
        let mut changes = Vec::new();

        slots.reconcile(100, capacity, window(0, 5), &mut changes);

        assert_eq!(capacity, 5);
        assert_eq!(slots.active_slot_count(), 5);
        assert_eq!(slots.window(), window(0, 5));
    }

    #[test]
    fn boundary_windows_backfill_to_slot_capacity() {
        let mut slots = UiVirtualListSlotMap::default();
        let mut changes = Vec::new();

        slots.reconcile(100, 7, window(0, 5), &mut changes);
        assert_eq!(slots.window(), window(0, 7));
        assert_eq!(slots.active_slot_count(), 7);

        slots.reconcile(100, 7, window(96, 100), &mut changes);
        assert_eq!(slots.window(), window(93, 100));
        assert_eq!(slots.active_slot_count(), 7);
        assert_eq!(changes.len(), 7);
    }

    #[test]
    fn one_row_scroll_rebinds_only_one_boundary_slot() {
        let mut slots = UiVirtualListSlotMap::default();
        let mut changes = Vec::new();
        slots.reconcile(100, 6, window(10, 16), &mut changes);

        changes.clear();
        slots.reconcile(100, 6, window(11, 17), &mut changes);

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].previous_logical_index, Some(10));
        assert_eq!(changes[0].logical_index, Some(16));
        assert_eq!(slots.active_slot_count(), 6);
    }

    #[test]
    fn large_seek_rebinds_at_most_the_slot_capacity() {
        let mut slots = UiVirtualListSlotMap::default();
        let mut changes = Vec::new();
        slots.reconcile(100_000, 8, window(0, 8), &mut changes);

        changes.clear();
        slots.reconcile(100_000, 8, window(50_000, 50_008), &mut changes);

        assert_eq!(changes.len(), slots.slot_count());
        assert!(changes.len() <= 8);
        assert_eq!(slots.window(), window(50_000, 50_008));
    }

    #[test]
    fn model_shrink_clears_out_of_range_assignments() {
        let mut slots = UiVirtualListSlotMap::default();
        let mut changes = Vec::new();
        slots.reconcile(100, 6, window(90, 96), &mut changes);

        changes.clear();
        slots.reconcile(3, 6, window(0, 6), &mut changes);

        assert_eq!(slots.logical_count(), 3);
        assert_eq!(slots.slot_count(), 3);
        assert_eq!(slots.active_slot_count(), 3);
        assert!((0..slots.slot_count()).all(|slot_index| {
            slots
                .logical_index_for_slot(slot_index)
                .is_some_and(|logical_index| logical_index < 3)
        }));
    }

    #[test]
    fn identical_reconcile_preserves_generation_and_emits_no_changes() {
        let mut slots = UiVirtualListSlotMap::default();
        let mut changes = Vec::new();
        slots.reconcile(100, 6, window(10, 16), &mut changes);
        let generation = slots.generation();

        slots.reconcile(100, 6, window(10, 16), &mut changes);

        assert!(changes.is_empty());
        assert_eq!(slots.generation(), generation);
    }

    fn window(first_visible: usize, last_visible_exclusive: usize) -> UiVirtualListWindow {
        UiVirtualListWindow {
            first_visible,
            last_visible_exclusive,
        }
    }
}
