use zircon_runtime_interface::ui::layout::UiVirtualListWindow;

/// A changed physical row slot and its previous/next logical assignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiVirtualListSlotChange {
    pub slot_index: usize,
    pub previous_logical_index: Option<usize>,
    pub logical_index: Option<usize>,
}

/// Retains a bounded physical-slot mapping for a logical fixed-extent list window.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct UiVirtualListSlotMap {
    slot_logical_indices: Vec<Option<usize>>,
    logical_count: usize,
    window: UiVirtualListWindow,
    generation: u64,
}

impl Clone for UiVirtualListSlotMap {
    fn clone(&self) -> Self {
        Self {
            slot_logical_indices: self.slot_logical_indices.clone(),
            logical_count: self.logical_count,
            window: self.window,
            generation: self.generation,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.slot_logical_indices
            .clone_from(&source.slot_logical_indices);
        self.logical_count = source.logical_count;
        self.window = source.window;
        self.generation = source.generation;
    }
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

/// Returns the requested live row-slot count for a fixed-extent viewport.
///
/// This calculation deliberately preserves the full request. The surface owner must reject a
/// result above its physical-slot budget before passing it to [`UiVirtualListSlotMap::reconcile`].
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
#[path = "tests/materialization.rs"]
mod tests;
