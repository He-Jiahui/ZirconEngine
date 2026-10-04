//! Regression drafts exercise the safe authoritative registry/release core.
//! No test invokes the unsafe public free function with an invalid descriptor.
use super::registry::{lock_registry, Registry};
use super::*;
use std::num::NonZeroU64;
use std::sync::{Arc, Barrier, Mutex};

#[test]
fn exact_allocation_consumes_once_and_copied_descriptor_replay_is_rejected() {
    let mut registry = Registry::new();
    let buffer = registry.register(b"retained payload".to_vec()).unwrap();
    let copied = buffer;
    let allocation = registry.consume(buffer).expect("registered exact tuple");
    assert_eq!(allocation, b"retained payload");
    drop(allocation);
    assert!(registry.consume(copied).is_none());
}

#[test]
fn wrong_id_address_length_capacity_preserve_both_real_owners() {
    let mut registry = Registry::new();
    let first = registry.register(b"first".to_vec()).unwrap();
    let second = registry.register(b"second".to_vec()).unwrap();
    let mut cases = [first; 4];
    cases[0].owner_token = second.owner_token;
    cases[1].data = second.data;
    cases[2].len -= 1;
    cases[3].capacity += 1;
    for malformed in cases {
        assert!(registry.consume(malformed).is_none());
    }
    assert_eq!(registry.consume(first).unwrap(), b"first");
    assert_eq!(registry.consume(second).unwrap(), b"second");
}

#[test]
fn legacy_reversible_token_and_unknown_id_have_no_allocation_authority() {
    let mut registry = Registry::new();
    let buffer = registry.register(b"opaque".to_vec()).unwrap();
    let mut forged = buffer;
    forged.owner_token = 0x5a17_c0de_f11e_d00d
        ^ buffer.data as usize as u64
        ^ ((buffer.len as u64) << 7)
        ^ ((buffer.capacity as u64) << 17);
    // Deliberately ensure this is not the issued ID: ID guessing is not a
    // cryptographic boundary; authority requires the exact whole descriptor.
    if forged.owner_token == buffer.owner_token {
        forged.owner_token ^= 1;
    }
    assert!(registry.consume(forged).is_none());
    forged.owner_token = u64::MAX;
    assert!(registry.consume(forged).is_none());
    assert_eq!(registry.consume(buffer).unwrap(), b"opaque");
}

#[test]
fn ids_are_nonzero_and_not_reused_after_consumption() {
    let mut registry = Registry::new();
    let first = registry.register(b"first".to_vec()).unwrap();
    drop(registry.consume(first).unwrap());
    let next = registry.register(b"next".to_vec()).unwrap();
    assert_ne!(first.owner_token, 0);
    assert!(next.owner_token > first.owner_token);
    assert!(registry.consume(first).is_none());
    assert_eq!(registry.consume(next).unwrap(), b"next");
}

#[test]
fn checked_last_id_exhaustion_returns_original_unpublished_vec() {
    let mut registry = Registry::new();
    registry.next_id = NonZeroU64::new(u64::MAX);
    let last = registry.register(b"last".to_vec()).unwrap();
    assert_eq!(last.owner_token, u64::MAX);
    let original = b"still owned".to_vec();
    let address = original.as_ptr();
    let capacity = original.capacity();
    let error = registry.register(original).unwrap_err();
    assert_eq!(
        error.kind(),
        NativePluginOwnedBytesErrorKind::AllocationIdsExhausted
    );
    assert_eq!(
        error.status().code,
        super::super::ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
    );
    let recovered = error.into_bytes();
    assert_eq!(recovered.as_ptr(), address);
    assert_eq!(recovered.capacity(), capacity);
    assert_eq!(recovered, b"still owned");
    assert_eq!(registry.consume(last).unwrap(), b"last");
    assert!(registry.consume(last).is_none());
}

#[test]
fn registry_reservation_failure_returns_owner_without_issuing_id() {
    let mut registry = Registry::new();
    let original = b"reservation retained".to_vec();
    let address = original.as_ptr();
    let capacity = original.capacity();
    // Exact same failure mapping as production, with a deterministic capacity
    // overflow from the real HashMap::try_reserve; no allocator OOM/UB injection.
    let error = registry
        .register_with_reservation(original, usize::MAX)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        NativePluginOwnedBytesErrorKind::RegistryCapacity
    );
    let recovered = error.into_bytes();
    assert_eq!(recovered.as_ptr(), address);
    assert_eq!(recovered.capacity(), capacity);
    assert_eq!(recovered, b"reservation retained");
    let next = registry.register(b"next".to_vec()).unwrap();
    assert_eq!(next.owner_token, 1);
    drop(registry.consume(next).unwrap());
}

#[test]
fn concurrent_copies_remove_one_real_allocation() {
    let registry = Arc::new(Mutex::new(Registry::new()));
    let buffer = lock_registry(&registry)
        .register(b"concurrent".to_vec())
        .unwrap();
    let tuple = (
        buffer.data as usize,
        buffer.len,
        buffer.capacity,
        buffer.owner_token,
    );
    let barrier = Arc::new(Barrier::new(8));
    let successes = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let registry = Arc::clone(&registry);
                let barrier = Arc::clone(&barrier);
                scope.spawn(move || {
                    let copied = super::super::NativePluginOwnedByteBufferV3 {
                        data: tuple.0 as *mut u8,
                        len: tuple.1,
                        capacity: tuple.2,
                        owner_token: tuple.3,
                        free: None,
                    };
                    barrier.wait();
                    let allocation = { lock_registry(&registry).consume(copied) };
                    if let Some(allocation) = allocation {
                        assert_eq!(allocation, b"concurrent");
                        drop(allocation);
                        1
                    } else {
                        0
                    }
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .sum::<usize>()
    });
    assert_eq!(successes, 1);
    assert!(lock_registry(&registry).consume(buffer).is_none());
}

#[test]
fn empty_and_retained_capacity_empty_have_distinct_authority() {
    let mut registry = Registry::new();
    let empty = registry.register(Vec::new()).unwrap();
    assert!(empty.data.is_null());
    assert_eq!(empty.owner_token, 0);
    assert!(registry.consume(empty).is_none());
    let retained = registry.register(Vec::with_capacity(16)).unwrap();
    assert_eq!(retained.len, 0);
    assert!(retained.capacity >= 16);
    assert_ne!(retained.owner_token, 0);
    drop(registry.consume(retained).unwrap());
    assert!(registry.consume(retained).is_none());
}

#[test]
fn safe_release_core_rejects_shapes_without_dereferencing_foreign_data() {
    use super::super::{
        NativePluginOwnedByteBufferV3 as Buffer, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR as ERROR,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK as OK,
    };
    assert_eq!(release(Buffer::empty()).code, OK);
    let owned = register(b"release core".to_vec()).unwrap();
    let mut malformed = owned;
    malformed.len = malformed.capacity + 1;
    assert_eq!(release(malformed).code, ERROR);
    malformed = owned;
    malformed.data = std::ptr::null_mut();
    assert_eq!(release(malformed).code, ERROR);
    malformed = owned;
    malformed.capacity = 0;
    assert_eq!(release(malformed).code, ERROR);
    malformed = owned;
    malformed.owner_token = 0;
    assert_eq!(release(malformed).code, ERROR);
    assert_eq!(release(owned).code, OK);
    assert_eq!(release(owned).code, ERROR);
}

#[test]
fn poisoned_mutex_retains_owner_and_does_not_recycle_id() {
    let registry = Mutex::new(Registry::new());
    let buffer = lock_registry(&registry)
        .register(b"poison survivor".to_vec())
        .unwrap();
    std::thread::scope(|scope| {
        assert!(scope
            .spawn(|| {
                let _guard = registry.lock().unwrap();
                panic!("controlled lock poison without registry mutation");
            })
            .join()
            .is_err());
    });
    assert!(registry.is_poisoned());
    let bytes = { lock_registry(&registry).consume(buffer) }.unwrap();
    assert_eq!(bytes, b"poison survivor");
    drop(bytes);
    assert!(lock_registry(&registry).consume(buffer).is_none());
    let next = lock_registry(&registry).register(b"next".to_vec()).unwrap();
    assert_ne!(next.owner_token, buffer.owner_token);
    drop(lock_registry(&registry).consume(next).unwrap());
}
