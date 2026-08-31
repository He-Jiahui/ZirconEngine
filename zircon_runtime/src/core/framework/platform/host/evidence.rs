use std::sync::Arc;

use super::{PlatformHostEvidenceError, PlatformHostObservedCapabilities};

pub const PLATFORM_HOST_BACKEND_VERSION_MAX_BYTES: usize = 128;

/// Bounded observations that justify a host's current Ready or Degraded fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlatformHostEvidence {
    observed_capabilities: PlatformHostObservedCapabilities,
    backend_version: Option<Arc<str>>,
}

impl PlatformHostEvidence {
    pub const fn new(observed_capabilities: PlatformHostObservedCapabilities) -> Self {
        Self {
            observed_capabilities,
            backend_version: None,
        }
    }

    pub fn with_backend_version(
        mut self,
        backend_version: impl Into<Arc<str>>,
    ) -> Result<Self, PlatformHostEvidenceError> {
        let backend_version = backend_version.into();
        if backend_version.is_empty() {
            return Err(PlatformHostEvidenceError::EmptyBackendVersion);
        }
        if backend_version.len() > PLATFORM_HOST_BACKEND_VERSION_MAX_BYTES {
            return Err(PlatformHostEvidenceError::BackendVersionTooLong {
                actual: backend_version.len(),
                maximum: PLATFORM_HOST_BACKEND_VERSION_MAX_BYTES,
            });
        }
        self.backend_version = Some(backend_version);
        Ok(self)
    }

    pub const fn observed_capabilities(&self) -> PlatformHostObservedCapabilities {
        self.observed_capabilities
    }

    pub fn backend_version(&self) -> Option<&str> {
        self.backend_version.as_deref()
    }
}
