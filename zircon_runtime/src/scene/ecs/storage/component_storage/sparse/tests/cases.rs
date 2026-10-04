use std::collections::HashMap;
use std::mem::size_of;

use super::*;

// 测试同时覆盖三种 locator 表示、代际校验、swap-remove 修复和随机参考模型。
#[derive(Debug, PartialEq, Eq)]
struct SparseValue(u32);

fn insert(storage: &mut SparseComponentStorage, entity: InternalEntity, value: u32) {
    assert!(storage
        .insert(entity, Box::new(SparseValue(value)), ChangeTick::new(1))
        .is_none());
}

#[test]
fn highest_valid_entity_index_allocates_one_locator_page() {
    let mut storage = SparseComponentStorage::default();
    let entity = InternalEntity::new(InternalEntity::INVALID_INDEX - 1, 7);

    insert(&mut storage, entity, 41);

    assert_eq!(storage.get::<SparseValue>(entity), Some(&SparseValue(41)));
    assert_eq!(storage.locator_page_count(), 1);
    assert_eq!(storage.locator_slot_capacity(), SPARSE_LOCATOR_PAGE_SLOTS);
    assert!(storage.locator_allocated_bytes() <= 16 * 1024);
}

#[test]
fn removing_the_last_row_retires_the_locator_hierarchy() {
    let mut storage = SparseComponentStorage::default();
    let entity = InternalEntity::new(1_000_000_000, 3);
    insert(&mut storage, entity, 9);
    assert!(storage.locator_allocated_bytes() > 0);

    assert!(storage.remove(entity).is_some());

    assert_eq!(storage.len(), 0);
    assert_eq!(storage.locator_page_count(), 0);
    assert_eq!(storage.locator_slot_capacity(), 0);
    assert_eq!(storage.locator_allocated_bytes(), 0);
}

#[test]
fn removing_one_row_keeps_the_shared_locator_page_alive() {
    let mut storage = SparseComponentStorage::default();
    let removed = InternalEntity::new(5, 1);
    let retained = InternalEntity::new(6, 2);
    insert(&mut storage, removed, 5);
    insert(&mut storage, retained, 6);

    assert!(storage.remove(removed).is_some());

    assert_eq!(storage.locator_page_count(), 1);
    assert_eq!(storage.get::<SparseValue>(retained), Some(&SparseValue(6)));
}

#[test]
fn density_bound_promotes_across_empty_low_pages() {
    let mut storage = SparseComponentStorage::default();
    let distant = SPARSE_LOCATOR_PAGE_SLOTS as u32 * 10;
    insert(&mut storage, InternalEntity::new(distant, 4), distant);

    assert_eq!(storage.locator_flat_prefix_slots(), 0);
    assert_eq!(
        storage.locator_flat_window_slots(),
        SPARSE_LOCATOR_PAGE_SLOTS
    );
    assert_eq!(storage.locator_sparse_page_count(), 0);

    insert(&mut storage, InternalEntity::new(0, 4), 0);
    assert_eq!(
        storage.locator_flat_prefix_slots(),
        SPARSE_LOCATOR_PAGE_SLOTS
    );
    assert_eq!(
        storage.locator_flat_window_slots(),
        SPARSE_LOCATOR_PAGE_SLOTS
    );
    assert_eq!(storage.locator_sparse_page_count(), 0);

    insert(&mut storage, InternalEntity::new(1, 4), 1);

    assert_eq!(
        storage.locator_flat_prefix_slots(),
        SPARSE_LOCATOR_PAGE_SLOTS * 11
    );
    assert_eq!(storage.locator_flat_window_slots(), 0);
    assert_eq!(storage.locator_sparse_page_count(), 0);
    assert_eq!(
        storage.get::<SparseValue>(InternalEntity::new(distant, 4)),
        Some(&SparseValue(distant))
    );
    assert_eq!(
        storage.get::<SparseValue>(InternalEntity::new(distant - 1, 4)),
        None
    );
}

#[test]
fn distant_page_promotes_only_when_the_global_density_bound_reaches_it() {
    let mut storage = SparseComponentStorage::default();
    let distant = SPARSE_LOCATOR_PAGE_SLOTS as u32 * 4;
    insert(&mut storage, InternalEntity::new(distant, 5), 15);

    assert_eq!(storage.locator_flat_prefix_slots(), 0);
    assert_eq!(storage.locator_flat_window_base(), distant);
    assert_eq!(
        storage.locator_flat_window_slots(),
        SPARSE_LOCATOR_PAGE_SLOTS
    );
    assert_eq!(storage.locator_sparse_page_count(), 0);

    insert(&mut storage, InternalEntity::new(0, 5), 0);

    assert_eq!(
        storage.locator_flat_prefix_slots(),
        SPARSE_LOCATOR_PAGE_SLOTS * 5
    );
    assert_eq!(storage.locator_flat_location_count(), 2);
    assert_eq!(storage.locator_flat_window_slots(), 0);
    assert_eq!(storage.locator_sparse_page_count(), 0);
    assert_eq!(
        storage.get::<SparseValue>(InternalEntity::new(distant, 5)),
        Some(&SparseValue(15))
    );
}

#[test]
fn growing_high_window_is_absorbed_when_the_zero_prefix_reaches_it() {
    let mut storage = SparseComponentStorage::default();
    for page in 10_000..10_064_u32 {
        let page_start = page * SPARSE_LOCATOR_PAGE_SLOTS as u32;
        for slot in 0..16_u32 {
            let index = page_start + slot;
            insert(&mut storage, InternalEntity::new(index, 8), index);
        }
    }
    assert_eq!(
        storage.locator_flat_window_slots(),
        SPARSE_LOCATOR_PAGE_SLOTS * 64
    );
    assert_eq!(storage.locator_sparse_page_count(), 0);

    for index in 0..1_536_u32 {
        insert(&mut storage, InternalEntity::new(index, 8), index);
    }

    assert_eq!(
        storage.locator_flat_prefix_slots(),
        SPARSE_LOCATOR_PAGE_SLOTS * 10_064
    );
    assert_eq!(storage.locator_sparse_page_count(), 0);
    assert_eq!(storage.locator_flat_window_slots(), 0);
    assert_eq!(storage.locator_sparse_directory_capacity(), 0);
}

#[test]
fn recreated_sparse_page_is_promoted_once_after_key_reinsertion() {
    let mut locator = SparseRowLocator::default();
    let page_20 = SPARSE_LOCATOR_PAGE_SLOTS as u32 * 20;
    let page_30 = SPARSE_LOCATOR_PAGE_SLOTS as u32 * 30;
    locator.insert(page_30, SparseRowLocation::new(3, 0));
    locator.insert(page_20, SparseRowLocation::new(3, 1));
    assert!(locator.remove(page_20).is_some());
    locator.insert(page_20, SparseRowLocation::new(4, 1));

    for index in 0..4_u32 {
        locator.insert(index, SparseRowLocation::new(5, index as usize + 2));
    }

    let promoted = locator.get(page_20).expect("recreated page is promoted");
    assert_eq!(promoted.generation(), 4);
    assert_eq!(promoted.dense_row(), 1);
    assert!(locator.flat_prefix_slots() >= SPARSE_LOCATOR_PAGE_SLOTS * 21);
}

#[test]
fn low_density_prefix_rebases_the_retained_cluster_as_a_flat_window() {
    let mut storage = SparseComponentStorage::default();
    for index in (0..262_144_u32).step_by(1_000) {
        insert(&mut storage, InternalEntity::new(index, 6), index);
    }
    assert!(storage.locator_flat_prefix_slots() >= 262_144);

    for index in (0..136_000_u32).step_by(1_000) {
        assert!(storage.remove(InternalEntity::new(index, 6)).is_some());
    }

    let retained = InternalEntity::new(262_000, 6);
    assert_eq!(
        storage.get::<SparseValue>(retained),
        Some(&SparseValue(262_000))
    );
    assert_eq!(storage.locator_flat_prefix_slots(), 0);
    assert!(storage.locator_flat_window_slots() <= 127_000);
    assert_eq!(storage.locator_sparse_page_count(), 0);
}

#[test]
fn low_density_high_window_trims_empty_leading_pages_before_demotion() {
    let mut locator = SparseRowLocator::default();
    let base = 4_000_000_u32;
    for (dense_row, index) in (base..base + 262_144).step_by(1_000).enumerate() {
        locator.insert(index, SparseRowLocation::new(7, dense_row));
    }
    assert!(locator.flat_window_slots() >= 262_144);

    for index in (base..base + 136_000).step_by(1_000) {
        assert!(locator.remove(index).is_some());
    }

    let retained = base + 262_000;
    assert_eq!(
        locator.get(retained).map(SparseRowLocation::generation),
        Some(7)
    );
    assert!(locator.flat_window_slots() <= 127_000);
    assert_eq!(locator.sparse_page_count(), 0);
}

#[test]
fn widely_separated_window_rows_demote_to_bounded_sparse_pages() {
    let mut locator = SparseRowLocator::default();
    let base = 4_000_000_u32;
    for (dense_row, index) in (base..base + 262_144).step_by(1_000).enumerate() {
        locator.insert(index, SparseRowLocation::new(8, dense_row));
    }
    for index in (base + 1_000..base + 262_000).step_by(1_000) {
        assert!(locator.remove(index).is_some());
    }

    assert!(locator.get(base).is_some());
    assert!(locator.get(base + 262_000).is_some());
    assert_eq!(locator.flat_window_slots(), 0);
    assert_eq!(locator.sparse_page_count(), 2);
}

#[test]
fn deleting_window_support_rechecks_an_under_dense_prefix() {
    let mut locator = SparseRowLocator::default();
    let high_base = 4_000_000_u32;
    for offset in 0..1_024_u32 {
        locator.insert(
            high_base + offset,
            SparseRowLocation::new(9, offset as usize),
        );
    }
    let low_outlier = 1_000_000_u32;
    locator.insert(low_outlier, SparseRowLocation::new(9, 1_024));
    assert!(locator.flat_prefix_slots() >= low_outlier as usize);
    assert!(locator.flat_window_slots() > 0);

    assert!(locator.remove(high_base).is_some());

    assert_eq!(locator.flat_prefix_slots(), 0);
    assert!(locator.get(low_outlier).is_some());
    assert_eq!(locator.sparse_page_count(), 1);
}

#[test]
fn deleting_prefix_support_rechecks_an_under_dense_window() {
    let mut locator = SparseRowLocator::default();
    for index in 0..3_000_u32 {
        locator.insert(index, SparseRowLocation::new(10, index as usize));
    }
    let high_start = 4_000_000_u32;
    let high_end = 5_000_000_u32;
    locator.insert(high_start, SparseRowLocation::new(10, 3_000));
    locator.insert(high_end, SparseRowLocation::new(10, 3_001));
    assert!(locator.flat_window_slots() >= 1_000_000);

    assert!(locator.remove(0).is_some());

    assert_eq!(locator.flat_window_slots(), 0);
    assert!(locator.get(high_start).is_some());
    assert!(locator.get(high_end).is_some());
    assert_eq!(locator.sparse_page_count(), 2);
}

#[test]
fn a_third_distant_cluster_remains_in_bounded_sparse_overflow() {
    let mut locator = SparseRowLocator::default();
    locator.insert(0, SparseRowLocation::new(1, 0));
    locator.insert(4_000_000, SparseRowLocation::new(1, 1));
    locator.insert(
        InternalEntity::INVALID_INDEX - 1,
        SparseRowLocation::new(1, 2),
    );

    assert_eq!(locator.flat_prefix_slots(), SPARSE_LOCATOR_PAGE_SLOTS);
    assert_eq!(locator.flat_window_slots(), SPARSE_LOCATOR_PAGE_SLOTS);
    assert_eq!(locator.sparse_page_count(), 1);
    for (index, dense_row) in [
        (0, 0),
        (4_000_000, 1),
        (InternalEntity::INVALID_INDEX - 1, 2),
    ] {
        let location = locator.get(index).expect("all three representations route");
        assert_eq!(location.generation(), 1);
        assert_eq!(location.dense_row(), dense_row);
    }
    assert!(locator.allocated_bytes() <= 24 * 1024);
}

#[test]
fn generation_checks_and_swap_remove_repair_survive_page_boundaries() {
    let mut storage = SparseComponentStorage::default();
    let first = InternalEntity::new(3, 1);
    let swapped = InternalEntity::new(0xFE_12_34_56, 8);
    insert(&mut storage, first, 10);
    insert(&mut storage, swapped, 20);

    assert!(storage.remove(first).is_some());

    assert_eq!(storage.get::<SparseValue>(swapped), Some(&SparseValue(20)));
    assert_eq!(
        storage.get::<SparseValue>(InternalEntity::new(swapped.index(), 7)),
        None
    );
    assert_eq!(storage.locator_page_count(), 1);
}

#[test]
fn packed_locator_uses_eight_bytes_per_allocated_slot() {
    assert_eq!(size_of::<Option<SparseRowLocation>>(), size_of::<u64>());
}

#[test]
fn locator_matches_a_reference_map_during_mixed_operations() {
    let mut locator = SparseRowLocator::default();
    let mut reference = HashMap::<u32, (u32, usize)>::new();
    let mut state = 0x9E37_79B9_u32;
    let mut operation_state = 0xD1B5_4A35_u32;
    let mut successful_removals = [0_usize; 4];

    for step in 0..20_000_usize {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        operation_state = operation_state
            .wrapping_mul(747_796_405)
            .wrapping_add(2_891_336_453);
        let cluster = step % 4;
        let index = match cluster {
            0 => state & 0x3FF,
            1 => 65_536 + (state & 0x1FFF),
            2 => 4_000_000 + (state & 0x3FFF),
            _ => InternalEntity::INVALID_INDEX - 1 - (state & 0x1FFF),
        };

        if operation_state >> 30 == 0 {
            let removed = locator
                .remove(index)
                .map(|location| (location.generation(), location.dense_row()));
            successful_removals[cluster] += usize::from(removed.is_some());
            assert_eq!(removed, reference.remove(&index));
        } else {
            let generation = state.rotate_left(11);
            let dense_row = step;
            let previous = locator
                .insert(index, SparseRowLocation::new(generation, dense_row))
                .map(|location| (location.generation(), location.dense_row()));
            assert_eq!(previous, reference.insert(index, (generation, dense_row)));
        }

        let actual = locator
            .get(index)
            .map(|location| (location.generation(), location.dense_row()));
        assert_eq!(actual, reference.get(&index).copied());
    }

    for (index, expected) in reference {
        let actual = locator
            .get(index)
            .map(|location| (location.generation(), location.dense_row()));
        assert_eq!(actual, Some(expected));
    }
    assert!(successful_removals.into_iter().all(|count| count > 0));
}
#[test]
fn sparse_shared_tracked_rows_keep_each_value_and_tick_separate() {
    let entities = [
        InternalEntity::new(3, 1),
        InternalEntity::new(0xFE_12_34_56, 8),
        InternalEntity::new(4, 2),
    ];
    let mut values = SparseComponentStorage::default();
    let mut labels = SparseComponentStorage::default();
    for (row, entity) in entities.into_iter().enumerate() {
        values.insert_with_ticks(
            entity,
            Box::new(SparseValue((row as u32 + 1) * 10)),
            ComponentTicks::new(ChangeTick::new(row as u64 + 2)),
        );
        labels.insert_with_ticks(
            entity,
            Box::new(format!("label-{row}")),
            ComponentTicks::new(ChangeTick::new(row as u64 + 5)),
        );
    }
    {
        // SAFETY: four loans select two distinct rows in each stable store.
        // Only unborrowed third rows are read, without any structure changes.
        let (first, first_ticks) =
            unsafe { values.get_mut_with_ticks_unchecked::<SparseValue>(entities[0]) }.unwrap();
        let (second, second_ticks) =
            unsafe { values.get_mut_with_ticks_unchecked::<SparseValue>(entities[1]) }.unwrap();
        let (first_label, first_label_ticks) =
            unsafe { labels.get_mut_with_ticks_unchecked::<String>(entities[0]) }.unwrap();
        let (second_label, second_label_ticks) =
            unsafe { labels.get_mut_with_ticks_unchecked::<String>(entities[1]) }.unwrap();
        second.0 += 2;
        first.0 += 1;
        second_label.push_str("-second");
        first_label.push_str("-first");
        second_ticks.set_changed(ChangeTick::new(20));
        first_ticks.set_changed(ChangeTick::new(10));
        second_label_ticks.set_changed(ChangeTick::new(40));
        first_label_ticks.set_changed(ChangeTick::new(30));
        assert_eq!((first.0, second.0), (11, 22));
        assert_eq!(
            (first_label.as_str(), second_label.as_str()),
            ("label-0-first", "label-1-second")
        );
        assert_eq!(
            values.get::<SparseValue>(entities[2]),
            Some(&SparseValue(30))
        );
        assert_eq!(
            values.ticks_for_type::<SparseValue>(entities[2]),
            Some(ComponentTicks::new(ChangeTick::new(4)))
        );
        assert_eq!(labels.get::<String>(entities[2]).unwrap(), "label-2");
        assert_eq!(
            labels.ticks_for_type::<String>(entities[2]),
            Some(ComponentTicks::new(ChangeTick::new(7)))
        );
    }
    assert_eq!(
        values.ticks(entities[0]).unwrap().added(),
        ChangeTick::new(2)
    );
    assert_eq!(
        values.ticks(entities[0]).unwrap().changed(),
        ChangeTick::new(10)
    );
    assert_eq!(
        values.ticks(entities[1]).unwrap().changed(),
        ChangeTick::new(20)
    );
    assert_eq!(
        labels.ticks(entities[0]).unwrap().changed(),
        ChangeTick::new(30)
    );
    assert_eq!(
        labels.ticks(entities[1]).unwrap().changed(),
        ChangeTick::new(40)
    );
    let stale = InternalEntity::new(entities[1].index(), 7);
    assert_eq!(values.ticks_for_type::<SparseValue>(stale), None);
    assert_eq!(values.ticks_for_type::<String>(entities[0]), None);
    // SAFETY: no row loans survive these checked failed lookups.
    assert!(unsafe { values.get_mut_with_ticks_unchecked::<SparseValue>(stale) }.is_none());
    assert!(unsafe { values.get_mut_with_ticks_unchecked::<String>(entities[0]) }.is_none());
}

#[test]
fn sparse_shared_eager_rows_preserve_exclusive_structure() {
    let entities = [
        InternalEntity::new(3, 1),
        InternalEntity::new(0xFE_12_34_56, 8),
        InternalEntity::new(4, 2),
    ];
    let mut storage = SparseComponentStorage::default();
    for (row, entity) in entities.into_iter().enumerate() {
        storage.insert_with_ticks(
            entity,
            Box::new(SparseValue((row as u32 + 1) * 10)),
            ComponentTicks::new(ChangeTick::new(row as u64 + 2)),
        );
    }
    {
        // SAFETY: rows 0/1 are uniquely selected and row 2 alone is read.
        // No replacement/removal/locator/allocation changes occur during loans.
        let first = unsafe {
            storage.get_mut_at_tick_unchecked::<SparseValue>(entities[0], ChangeTick::new(2))
        }
        .unwrap();
        let second = unsafe {
            storage.get_mut_at_tick_unchecked::<SparseValue>(entities[1], ChangeTick::new(9))
        }
        .unwrap();
        second.0 += 2;
        first.0 += 1;
        assert_eq!((first.0, second.0), (11, 22));
        assert_eq!(
            storage.ticks(entities[2]),
            Some(ComponentTicks::new(ChangeTick::new(4)))
        );
    }
    assert_eq!(
        storage.ticks(entities[0]),
        Some(ComponentTicks::new(ChangeTick::new(2)))
    );
    assert_eq!(
        storage.ticks(entities[1]).unwrap().added(),
        ChangeTick::new(3)
    );
    assert_eq!(
        storage.ticks(entities[1]).unwrap().changed(),
        ChangeTick::new(9)
    );
    // SAFETY: no row references are alive; this selected value/tick row is unique.
    let _ = unsafe {
        storage.get_mut_at_tick_unchecked::<SparseValue>(entities[2], ChangeTick::new(12))
    }
    .unwrap();
    assert_eq!(
        storage.ticks(entities[2]).unwrap().changed(),
        ChangeTick::new(12)
    );
    let previous = storage
        .insert(entities[1], Box::new(SparseValue(42)), ChangeTick::new(10))
        .unwrap();
    assert_eq!(
        *previous.downcast::<SparseValue>().unwrap(),
        SparseValue(22)
    );
    let removed = storage.remove(entities[0]).unwrap();
    assert_eq!(
        *removed.value.downcast::<SparseValue>().unwrap(),
        SparseValue(11)
    );
    assert_eq!(removed.ticks, ComponentTicks::new(ChangeTick::new(2)));
    assert_eq!(
        storage.get::<SparseValue>(entities[2]),
        Some(&SparseValue(30))
    );
    assert_eq!(
        storage.ticks(entities[2]).unwrap().changed(),
        ChangeTick::new(12)
    );
    assert_eq!(
        storage.get::<SparseValue>(InternalEntity::new(entities[2].index(), 1)),
        None
    );
    *storage.get_mut::<SparseValue>(entities[2]).unwrap() = SparseValue(31);
    assert_eq!(
        storage.get::<SparseValue>(entities[2]),
        Some(&SparseValue(31))
    );
    // SAFETY: no references remain. Preserve sparse's existing eager tick
    // update before a failed concrete downcast, without changing the payload.
    assert!(unsafe {
        storage.get_mut_at_tick_unchecked::<String>(entities[1], ChangeTick::new(15))
    }
    .is_none());
    assert_eq!(
        storage.ticks(entities[1]).unwrap().changed(),
        ChangeTick::new(15)
    );
    assert_eq!(
        storage.get::<SparseValue>(entities[1]),
        Some(&SparseValue(42))
    );
}
