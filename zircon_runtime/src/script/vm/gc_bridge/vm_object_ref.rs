use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmObjectId(u64);

impl VmObjectId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmGcRootToken(u64);

impl VmGcRootToken {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmGcRootRegistrationError {
    pub message: String,
}

impl VmGcRootRegistrationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for VmGcRootRegistrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for VmGcRootRegistrationError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VmObjectRefError {
    RegistrationFailed {
        object_id: VmObjectId,
        message: String,
    },
}

impl fmt::Display for VmObjectRefError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RegistrationFailed { object_id, message } => write!(
                formatter,
                "failed to register GC root for VM object {}: {message}",
                object_id.raw()
            ),
        }
    }
}

impl std::error::Error for VmObjectRefError {}

/// Backend-owned GC root table. Implementations keep VM pointers entirely internal.
pub trait VmGcRootRegistry: Send + Sync {
    fn register_gc_root(
        &self,
        object_id: VmObjectId,
    ) -> Result<VmGcRootToken, VmGcRootRegistrationError>;

    fn unregister_gc_root(&self, root_token: VmGcRootToken);
}

#[derive(Clone)]
pub struct VmObjectRef {
    lease: Arc<VmObjectRootLease>,
}

struct VmObjectRootLease {
    object_id: VmObjectId,
    root_token: VmGcRootToken,
    registry: Arc<dyn VmGcRootRegistry>,
}

impl VmObjectRef {
    pub fn new(
        object_id: VmObjectId,
        registry: Arc<dyn VmGcRootRegistry>,
    ) -> Result<Self, VmObjectRefError> {
        let root_token = registry.register_gc_root(object_id).map_err(|error| {
            VmObjectRefError::RegistrationFailed {
                object_id,
                message: error.message,
            }
        })?;
        Ok(Self {
            lease: Arc::new(VmObjectRootLease {
                object_id,
                root_token,
                registry,
            }),
        })
    }

    pub fn object_id(&self) -> VmObjectId {
        self.lease.object_id
    }

    pub fn root_token(&self) -> VmGcRootToken {
        self.lease.root_token
    }
}

impl fmt::Debug for VmObjectRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("VmObjectRef")
            .field("object_id", &self.object_id())
            .field("root_token", &self.root_token())
            .finish()
    }
}

impl Drop for VmObjectRootLease {
    fn drop(&mut self) {
        self.registry.unregister_gc_root(self.root_token);
    }
}

#[cfg(test)]
mod tests {
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
}
