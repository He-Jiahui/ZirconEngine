use crate::core::resource::ResourceId;

use super::super::slot_allocator::{ProbeCubemapSlotAllocation, ProbeCubemapSlotAllocator};

fn acquire_and_commit(
    allocator: &mut ProbeCubemapSlotAllocator,
    cubemap: ResourceId,
    revision: u64,
    prepare_epoch: u64,
) -> ProbeCubemapSlotAllocation {
    let allocation = allocator
        .acquire(cubemap, revision, prepare_epoch)
        .expect("ordinary probe allocation must be admitted");
    if allocation.requires_upload {
        allocator.commit(cubemap, revision, allocation.slot, prepare_epoch);
    }
    allocation
}

#[test]
fn render_probe_slot_allocator_rounds_capacity_to_power_of_two() {
    let allocator = ProbeCubemapSlotAllocator::new(3);

    assert_eq!(allocator.capacity(), 4);
    assert_eq!(allocator.physical_slot_count(), 5);
}

#[test]
fn render_probe_slot_allocator_evicts_lru_on_pressure() {
    let first = ResourceId::from_stable_label("probe:first");
    let second = ResourceId::from_stable_label("probe:second");
    let third = ResourceId::from_stable_label("probe:third");
    let mut allocator = ProbeCubemapSlotAllocator::new(2);

    let first_slot = acquire_and_commit(&mut allocator, first, 1, 1);
    let second_slot = acquire_and_commit(&mut allocator, second, 1, 2);
    let touched_first = allocator
        .acquire(first, 1, 3)
        .expect("ready probe touch must be admitted");
    let third_slot = acquire_and_commit(&mut allocator, third, 1, 4);

    assert_eq!(first_slot.slot, touched_first.slot);
    assert!(!touched_first.requires_upload);
    assert_eq!(third_slot.slot, second_slot.slot);
    assert_eq!(third_slot.evicted, Some(second));
    assert!(third_slot.requires_upload);
    assert!(allocator.available(second, 1, 5).is_none());
    assert_eq!(
        allocator.available(first, 1, 5).map(|slot| slot.slot),
        Some(first_slot.slot)
    );
}

#[test]
fn render_probe_slot_allocator_reuploads_changed_revision_without_eviction() {
    let cubemap = ResourceId::from_stable_label("probe:revision");
    let mut allocator = ProbeCubemapSlotAllocator::new(2);

    let first = acquire_and_commit(&mut allocator, cubemap, 4, 1);
    let unchanged = allocator
        .acquire(cubemap, 4, 2)
        .expect("unchanged probe must be admitted");
    let changed = allocator
        .acquire(cubemap, 5, 3)
        .expect("changed probe must be admitted");

    assert!(first.requires_upload);
    assert!(!unchanged.requires_upload);
    assert!(changed.requires_upload);
    assert_eq!(changed.slot, first.slot);
    assert_eq!(changed.evicted, None);
}

#[test]
fn render_probe_slot_allocator_reuses_only_current_epoch_pending_uploads() {
    let cubemap = ResourceId::from_stable_label("probe:transaction");
    let mut allocator = ProbeCubemapSlotAllocator::new(2);

    let first = allocator
        .acquire(cubemap, 4, 7)
        .expect("first pending upload must be admitted");
    assert!(first.requires_upload);
    assert!(allocator.available(cubemap, 4, 7).is_some());
    assert!(allocator.available(cubemap, 4, 8).is_none());

    let same_prepare = allocator
        .acquire(cubemap, 4, 7)
        .expect("same-epoch upload must be admitted");
    assert!(!same_prepare.requires_upload);

    let retry = allocator
        .acquire(cubemap, 4, 8)
        .expect("next-epoch retry must be admitted");
    assert!(retry.requires_upload);
    allocator.commit(cubemap, 4, first.slot, 7);
    assert!(allocator.available(cubemap, 4, 9).is_none());

    allocator.commit(cubemap, 4, retry.slot, 8);

    assert!(allocator.available(cubemap, 4, 9).is_some());
}

#[test]
fn render_probe_slot_allocator_preserves_lru_order_across_repeated_touches() {
    let first = ResourceId::from_stable_label("probe:lru:first");
    let second = ResourceId::from_stable_label("probe:lru:second");
    let third = ResourceId::from_stable_label("probe:lru:third");
    let fourth = ResourceId::from_stable_label("probe:lru:fourth");
    let fifth = ResourceId::from_stable_label("probe:lru:fifth");
    let sixth = ResourceId::from_stable_label("probe:lru:sixth");
    let mut allocator = ProbeCubemapSlotAllocator::new(3);

    let first_slot = acquire_and_commit(&mut allocator, first, 1, 1);
    let second_slot = acquire_and_commit(&mut allocator, second, 1, 2);
    let third_slot = acquire_and_commit(&mut allocator, third, 1, 3);
    let fourth_slot = acquire_and_commit(&mut allocator, fourth, 1, 4);
    assert_eq!(
        allocator
            .acquire(first, 1, 5)
            .expect("first touch must be admitted")
            .slot,
        first_slot.slot
    );
    assert_eq!(
        allocator
            .acquire(second, 1, 6)
            .expect("second touch must be admitted")
            .slot,
        second_slot.slot
    );

    let fifth_slot = acquire_and_commit(&mut allocator, fifth, 1, 7);
    assert_eq!(fifth_slot.evicted, Some(third));
    assert_eq!(fifth_slot.slot, third_slot.slot);
    assert_eq!(
        allocator
            .acquire(first, 1, 8)
            .expect("repeated first touch must be admitted")
            .slot,
        first_slot.slot
    );

    let sixth_slot = acquire_and_commit(&mut allocator, sixth, 1, 9);
    assert_eq!(sixth_slot.evicted, Some(fourth));
    assert_eq!(sixth_slot.slot, fourth_slot.slot);
    assert!(allocator.available(third, 1, 10).is_none());
    assert!(allocator.available(fourth, 1, 10).is_none());
    assert!(allocator.available(first, 1, 10).is_some());
    assert!(allocator.available(second, 1, 10).is_some());
    assert!(allocator.available(fifth, 1, 10).is_some());
    assert!(allocator.available(sixth, 1, 10).is_some());
}

#[test]
fn capture_revision_change_preserves_last_good_slot_until_cancel() {
    let cubemap = ResourceId::from_stable_label("probe:capture-revision-cancel");
    let mut allocator = ProbeCubemapSlotAllocator::new(1);
    let ready = acquire_and_commit(&mut allocator, cubemap, 1, 1);

    let reservation = allocator
        .reserve_for_capture(cubemap, 2, 2)
        .expect("ready target refresh must reserve the physical spare");

    assert_ne!(reservation.slot(), ready.slot);
    assert_eq!(
        allocator.available(cubemap, 1, 3).map(|slot| slot.slot),
        Some(ready.slot)
    );
    assert!(allocator.available(cubemap, 2, 3).is_none());
    let visible_during_refresh = allocator
        .acquire(cubemap, 2, 3)
        .expect("changed-revision capture must keep last-good visible");
    assert!(!visible_during_refresh.requires_upload);
    assert_eq!(visible_during_refresh.slot, ready.slot);

    allocator.cancel(reservation);

    assert_eq!(
        allocator.available(cubemap, 1, 4).map(|slot| slot.slot),
        Some(ready.slot)
    );
    assert!(allocator.available(cubemap, 2, 4).is_none());
}

#[test]
fn capture_commit_atomically_rotates_the_physical_spare() {
    let cubemap = ResourceId::from_stable_label("probe:capture-revision-commit");
    let mut allocator = ProbeCubemapSlotAllocator::new(1);
    let ready = acquire_and_commit(&mut allocator, cubemap, 1, 1);
    let replacement = allocator
        .reserve_for_capture(cubemap, 2, 2)
        .expect("ready target refresh must reserve the physical spare");

    allocator.commit(
        replacement.cubemap(),
        replacement.revision(),
        replacement.slot(),
        replacement.prepare_epoch(),
    );

    assert!(allocator.available(cubemap, 1, 3).is_none());
    assert_eq!(
        allocator.available(cubemap, 2, 3).map(|slot| slot.slot),
        Some(replacement.slot())
    );

    let next = allocator
        .reserve_for_capture(cubemap, 3, 4)
        .expect("committed replacement must release the former ready slot");
    assert_eq!(next.slot(), ready.slot);
}

#[test]
fn same_revision_capture_uses_the_spare_and_cancel_keeps_ready_texels() {
    let cubemap = ResourceId::from_stable_label("probe:capture-same-revision");
    let mut allocator = ProbeCubemapSlotAllocator::new(1);
    let ready = acquire_and_commit(&mut allocator, cubemap, 7, 1);

    let reservation = allocator
        .reserve_for_capture(cubemap, 7, 2)
        .expect("same-revision refresh must reserve the physical spare");

    assert_ne!(reservation.slot(), ready.slot);
    assert_eq!(
        allocator.available(cubemap, 7, 3).map(|slot| slot.slot),
        Some(ready.slot)
    );
    allocator.cancel(reservation);
    assert_eq!(
        allocator.available(cubemap, 7, 4).map(|slot| slot.slot),
        Some(ready.slot)
    );
}

#[test]
fn full_logical_capacity_allows_refresh_but_rejects_a_new_capture_target() {
    let first = ResourceId::from_stable_label("probe:capture-full:first");
    let second = ResourceId::from_stable_label("probe:capture-full:second");
    let third = ResourceId::from_stable_label("probe:capture-full:third");
    let mut allocator = ProbeCubemapSlotAllocator::new(2);
    acquire_and_commit(&mut allocator, first, 1, 1);
    acquire_and_commit(&mut allocator, second, 1, 2);

    let refresh = allocator
        .reserve_for_capture(first, 2, 3)
        .expect("physical spare must remain available for an existing target");
    allocator.cancel(refresh);

    assert!(allocator.reserve_for_capture(third, 1, 4).is_none());
    assert!(allocator.available(first, 1, 5).is_some());
    assert!(allocator.available(second, 1, 5).is_some());
}

#[test]
fn capture_allocator_admits_only_one_physical_transaction() {
    let first = ResourceId::from_stable_label("probe:capture-single:first");
    let second = ResourceId::from_stable_label("probe:capture-single:second");
    let mut allocator = ProbeCubemapSlotAllocator::new(2);

    let reservation = allocator
        .reserve_for_capture(first, 1, 1)
        .expect("first capture transaction must be admitted");
    assert!(allocator.reserve_for_capture(second, 1, 2).is_none());

    allocator.cancel(reservation);
    assert!(allocator.reserve_for_capture(second, 1, 3).is_some());
}

#[test]
fn pending_new_capture_target_cannot_be_materialized_by_ordinary_upload() {
    let cubemap = ResourceId::from_stable_label("probe:capture-new-target-owner");
    let mut allocator = ProbeCubemapSlotAllocator::new(2);
    let reservation = allocator
        .reserve_for_capture(cubemap, 1, 1)
        .expect("new capture target must reserve one logical and physical slot");

    assert!(allocator.acquire(cubemap, 1, 2).is_none());

    allocator.cancel(reservation);
    assert!(allocator.acquire(cubemap, 1, 3).is_some());
}

#[test]
fn ordinary_pressure_cannot_evict_the_capture_target_last_good_entry() {
    let captured = ResourceId::from_stable_label("probe:capture-protected");
    let competing = ResourceId::from_stable_label("probe:capture-competing");
    let mut allocator = ProbeCubemapSlotAllocator::new(1);
    let ready = acquire_and_commit(&mut allocator, captured, 1, 1);
    let reservation = allocator
        .reserve_for_capture(captured, 2, 2)
        .expect("existing target refresh must reserve the spare");

    assert!(allocator.acquire(competing, 1, 3).is_none());
    assert_eq!(
        allocator.available(captured, 1, 3).map(|slot| slot.slot),
        Some(ready.slot)
    );

    allocator.cancel(reservation);
}
