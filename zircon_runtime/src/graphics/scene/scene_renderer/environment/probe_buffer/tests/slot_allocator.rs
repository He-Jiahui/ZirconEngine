use crate::core::resource::ResourceId;

use super::super::slot_allocator::ProbeCubemapSlotAllocator;

#[test]
fn render_probe_slot_allocator_rounds_capacity_to_power_of_two() {
    let allocator = ProbeCubemapSlotAllocator::new(3);

    assert_eq!(allocator.capacity(), 4);
}

#[test]
fn render_probe_slot_allocator_evicts_lru_on_pressure() {
    let first = ResourceId::from_stable_label("probe:first");
    let second = ResourceId::from_stable_label("probe:second");
    let third = ResourceId::from_stable_label("probe:third");
    let mut allocator = ProbeCubemapSlotAllocator::new(2);

    let first_slot = allocator.acquire(first, 1);
    let second_slot = allocator.acquire(second, 1);
    let touched_first = allocator.acquire(first, 1);
    let third_slot = allocator.acquire(third, 1);

    assert_eq!(first_slot.slot, touched_first.slot);
    assert!(!touched_first.requires_upload);
    assert_eq!(third_slot.slot, second_slot.slot);
    assert_eq!(third_slot.evicted, Some(second));
    assert!(third_slot.requires_upload);
    assert!(allocator.get(second).is_none());
    assert_eq!(
        allocator.get(first).map(|slot| slot.slot),
        Some(first_slot.slot)
    );
}

#[test]
fn render_probe_slot_allocator_reuploads_changed_revision_without_eviction() {
    let cubemap = ResourceId::from_stable_label("probe:revision");
    let mut allocator = ProbeCubemapSlotAllocator::new(2);

    let first = allocator.acquire(cubemap, 4);
    let unchanged = allocator.acquire(cubemap, 4);
    let changed = allocator.acquire(cubemap, 5);

    assert!(first.requires_upload);
    assert!(!unchanged.requires_upload);
    assert!(changed.requires_upload);
    assert_eq!(changed.slot, first.slot);
    assert_eq!(changed.evicted, None);
}
