use std::sync::Arc;

use super::{WgpuUiImageAllocationLedger, WgpuUiImageAllocationSet};

#[test]
fn registry_eviction_remains_budgeted_until_the_surface_pin_drops() {
    let ledger = Arc::new(WgpuUiImageAllocationLedger::default());
    let registry_pin = ledger
        .try_allocate_for_test(64, 64)
        .expect("allocation fits");
    let surface_pin = registry_pin.surface_pin();

    drop(registry_pin);
    let pinned = ledger.stats();
    assert_eq!(pinned.unique_allocation_bytes, 64);
    assert_eq!(pinned.registry_evicted_pinned_bytes, 64);
    assert!(ledger.try_allocate_for_test(1, 64).is_none());

    drop(surface_pin);
    let released = ledger.stats();
    assert_eq!(released.unique_allocation_bytes, 0);
    assert_eq!(released.registry_evicted_pinned_bytes, 0);
    assert_eq!(released.eviction_completion_count, 1);
}

#[test]
fn in_flight_set_releases_allocations_only_after_completion_guard_drops() {
    let ledger = Arc::new(WgpuUiImageAllocationLedger::default());
    let registry_pin = ledger
        .try_allocate_for_test(64, 64)
        .expect("allocation fits");
    let surface_pin = registry_pin.surface_pin();
    let allocation_set = WgpuUiImageAllocationSet::from_surface_pins(vec![surface_pin]);
    let in_flight = allocation_set.begin_in_flight().expect("non-empty set");

    drop(registry_pin);
    drop(allocation_set);
    assert_eq!(ledger.stats().in_flight_present_pin_count, 1);
    assert_eq!(ledger.stats().unique_allocation_bytes, 64);

    drop(in_flight);
    assert_eq!(ledger.stats().in_flight_present_pin_count, 0);
    assert_eq!(ledger.stats().unique_allocation_bytes, 0);
}

#[test]
fn only_registry_owned_allocations_are_immediately_releasable() {
    let ledger = Arc::new(WgpuUiImageAllocationLedger::default());
    let registry_pin = ledger
        .try_allocate_for_test(64, 64)
        .expect("allocation fits");
    assert!(registry_pin.is_exclusively_registry_owned());

    let surface_pin = registry_pin.surface_pin();
    assert!(!registry_pin.is_exclusively_registry_owned());
    drop(surface_pin);
    assert!(registry_pin.is_exclusively_registry_owned());
}
