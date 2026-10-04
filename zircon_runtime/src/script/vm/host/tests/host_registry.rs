use std::panic::{catch_unwind, AssertUnwindSafe};

use super::*;

#[test]
fn validity_check_does_not_clone_the_capability_record() {
    let source = include_str!("../host_registry.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;
    let function = source.split("pub fn is_valid").nth(1).unwrap();
    let function = function.split("\n    }").next().unwrap();

    assert!(function.contains("slot.record.is_some()"));
    assert!(!function.contains("self.resolve(handle)"));
}

struct PanickingLabel;

impl From<PanickingLabel> for String {
    fn from(_: PanickingLabel) -> Self {
        panic!("intentional label conversion panic");
    }
}

#[test]
fn dead_host_object_access_returns_error_not_ub() {
    let registry = HostRegistry::default();
    let handle = registry.register_capability("test.capability").unwrap();
    registry.revoke(handle).unwrap();

    assert!(matches!(
        registry.resolve(handle),
        Err(HostRegistryError::GenerationMismatch { index, .. })
            if index == handle.index()
    ));
    assert!(!registry.is_valid(handle));
}

#[test]
fn stale_handle_remains_invalid_after_slot_reuse() {
    let registry = HostRegistry::default();
    let stale = registry.register_capability("first").unwrap();
    registry.revoke(stale).unwrap();
    let current = registry.register_capability("second").unwrap();

    assert_eq!(current.index(), stale.index());
    assert_eq!(current.generation(), stale.generation() + 1);
    assert!(matches!(
        registry.resolve(stale),
        Err(HostRegistryError::GenerationMismatch { .. })
    ));
    assert_eq!(registry.resolve(current).unwrap().label, "second");
}

#[test]
fn forged_current_generation_for_vacant_slot_is_rejected_as_vacant() {
    let registry = HostRegistry::default();
    let handle = registry.register_capability("first").unwrap();
    registry.revoke(handle).unwrap();
    let vacant = HostHandle::from_parts(handle.index(), handle.generation() + 1);

    assert!(matches!(
        registry.resolve(vacant),
        Err(HostRegistryError::VacantSlot { index, .. }) if index == handle.index()
    ));
}

#[test]
fn generation_exhaustion_keeps_live_record_valid() {
    let registry = HostRegistry::default();
    let handle = registry.register_capability("last-generation").unwrap();
    {
        let mut state = registry.lock_state();
        state.slots[handle.index() as usize].generation = u32::MAX;
        state.slots[handle.index() as usize]
            .record
            .as_mut()
            .unwrap()
            .handle = HostHandle::from_parts(handle.index(), u32::MAX);
    }
    let exhausted = HostHandle::from_parts(handle.index(), u32::MAX);

    assert!(matches!(
        registry.revoke(exhausted),
        Err(HostRegistryError::GenerationExhausted { .. })
    ));
    assert!(registry.is_valid(exhausted));
}

#[test]
fn host_registry_accessors_recover_poisoned_handle_lock() {
    let registry = HostRegistry::default();

    let poison_result = catch_unwind(AssertUnwindSafe(|| {
        let _guard = registry.state.lock().unwrap();
        panic!("poison host handle registry");
    }));
    assert!(poison_result.is_err());

    let handle = registry.register_capability("test.capability").unwrap();
    assert!(registry.is_valid(handle));
    assert_eq!(
        registry.resolve(handle).unwrap(),
        HostCapabilityRecord {
            handle,
            label: "test.capability".to_string(),
        }
    );
    assert_eq!(registry.capabilities().len(), 1);
}

#[test]
fn preallocated_capability_snapshot_preserves_live_sorted_contract() {
    let registry = HostRegistry::default();
    let first = registry.register_capability("first").unwrap();
    let second = registry.register_capability("second").unwrap();
    let third = registry.register_capability("third").unwrap();
    registry.revoke(second).unwrap();
    let replacement = registry.register_capability("replacement").unwrap();

    let mut expected = vec![
        registry.resolve(first).unwrap(),
        registry.resolve(third).unwrap(),
        registry.resolve(replacement).unwrap(),
    ];
    expected.sort_unstable_by_key(|record| record.handle.into_raw());

    assert_eq!(registry.capabilities(), expected);
    assert!(registry
        .capabilities()
        .iter()
        .all(|record| record.handle != second));
}

#[test]
fn panicking_label_conversion_does_not_consume_reusable_slot() {
    let registry = HostRegistry::default();
    let first = registry.register_capability("first").unwrap();
    registry.revoke(first).unwrap();

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = registry.register_capability(PanickingLabel);
    }));
    assert!(panic.is_err());

    let reused = registry.register_capability("reused").unwrap();
    assert_eq!(reused.index(), first.index());
    assert_eq!(reused.generation(), first.generation() + 1);
    assert_eq!(registry.capabilities().len(), 1);
}
