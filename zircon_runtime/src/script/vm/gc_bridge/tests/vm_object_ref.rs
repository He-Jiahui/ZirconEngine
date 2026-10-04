use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use super::*;

#[derive(Debug, Default)]
struct RecordingRootRegistry {
    fail_registration: AtomicBool,
    registrations: AtomicUsize,
    unregistrations: AtomicUsize,
}

impl VmGcRootRegistry for RecordingRootRegistry {
    fn register_gc_root(
        &self,
        object_id: VmObjectId,
    ) -> Result<VmGcRootToken, VmGcRootRegistrationError> {
        self.registrations.fetch_add(1, Ordering::SeqCst);
        if self.fail_registration.load(Ordering::SeqCst) {
            return Err(VmGcRootRegistrationError::new("root table is full"));
        }
        Ok(VmGcRootToken::new(object_id.raw() + 100))
    }

    fn unregister_gc_root(&self, _root_token: VmGcRootToken) {
        self.unregistrations.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn dropped_ref_unregisters_gc_root() {
    let registry = Arc::new(RecordingRootRegistry::default());
    let object_ref = VmObjectRef::new(VmObjectId::new(7), registry.clone()).unwrap();

    assert_eq!(object_ref.root_token(), VmGcRootToken::new(107));
    drop(object_ref);

    assert_eq!(registry.registrations.load(Ordering::SeqCst), 1);
    assert_eq!(registry.unregistrations.load(Ordering::SeqCst), 1);
}

#[test]
fn cloned_refs_share_one_root_lease() {
    let registry = Arc::new(RecordingRootRegistry::default());
    let first = VmObjectRef::new(VmObjectId::new(8), registry.clone()).unwrap();
    let second = first.clone();

    drop(first);
    assert_eq!(registry.unregistrations.load(Ordering::SeqCst), 0);
    drop(second);

    assert_eq!(registry.registrations.load(Ordering::SeqCst), 1);
    assert_eq!(registry.unregistrations.load(Ordering::SeqCst), 1);
}

#[test]
fn failed_registration_creates_no_live_ref() {
    let registry = Arc::new(RecordingRootRegistry::default());
    registry.fail_registration.store(true, Ordering::SeqCst);

    let error = VmObjectRef::new(VmObjectId::new(9), registry.clone()).unwrap_err();

    assert!(matches!(
        error,
        VmObjectRefError::RegistrationFailed { object_id, .. }
            if object_id == VmObjectId::new(9)
    ));
    assert_eq!(registry.registrations.load(Ordering::SeqCst), 1);
    assert_eq!(registry.unregistrations.load(Ordering::SeqCst), 0);
}

#[test]
fn last_ref_bounds_backend_registry_lifetime() {
    let registry = Arc::new(RecordingRootRegistry::default());
    let weak_registry = Arc::downgrade(&registry);
    let object_ref = VmObjectRef::new(VmObjectId::new(10), registry.clone()).unwrap();
    drop(registry);

    assert!(weak_registry.upgrade().is_some());
    drop(object_ref);
    assert!(weak_registry.upgrade().is_none());
}

#[test]
fn vm_object_ref_is_send_and_sync_without_exposing_vm_pointers() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<VmObjectRef>();
}
