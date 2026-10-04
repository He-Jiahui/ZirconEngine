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

#[test]
fn clone_from_reuses_slot_storage() {
    let mut source = UiVirtualListSlotMap::default();
    let mut target = UiVirtualListSlotMap::default();
    let mut changes = Vec::new();
    source.reconcile(100, 8, window(10, 18), &mut changes);
    target.reconcile(100, 8, window(0, 8), &mut changes);
    let pointer = target.slot_logical_indices.as_ptr();
    let capacity = target.slot_logical_indices.capacity();

    target.clone_from(&source);

    assert_eq!(target, source);
    assert_eq!(target.slot_logical_indices.as_ptr(), pointer);
    assert_eq!(target.slot_logical_indices.capacity(), capacity);
}

fn window(first_visible: usize, last_visible_exclusive: usize) -> UiVirtualListWindow {
    UiVirtualListWindow {
        first_visible,
        last_visible_exclusive,
    }
}
