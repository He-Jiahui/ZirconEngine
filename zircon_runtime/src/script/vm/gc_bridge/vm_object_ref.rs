//! VM 对象引用只暴露稳定 ID 和共享根租约；最后一个宿主引用释放才撤销后端 GC 根，避免脚本对象被宿主持有期间回收。
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
#[path = "tests/vm_object_ref.rs"]
mod tests;
