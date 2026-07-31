use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::platform::PreferenceStorageBackendKind;

use super::super::preferences::{
    PreferencePersistenceAdapter, PreferencePersistenceLimits, PreferenceStorageBackend,
    UnavailablePreferenceStorageBackend,
};

/// Same-domain driver slot that lets the process host install one platform backend.
pub struct PlatformDriver {
    preference_storage: Arc<PreferencePersistenceAdapter>,
    install_state: Mutex<PreferenceStorageBackendInstallState>,
}

impl PlatformDriver {
    pub fn install_preference_storage_backend(
        &self,
        backend: Arc<dyn PreferenceStorageBackend>,
    ) -> Result<(), PreferenceStorageBackendInstallError> {
        let requested = backend.backend_kind();
        let mut state = self.lock_install_state();
        let current = self.preference_storage.backend_kind();
        if requested == PreferenceStorageBackendKind::Unavailable {
            return Err(PreferenceStorageBackendInstallError::new(
                PreferenceStorageBackendInstallErrorKind::UnavailableBackend,
                current,
                requested,
            ));
        }
        if state.installed {
            return Err(PreferenceStorageBackendInstallError::new(
                PreferenceStorageBackendInstallErrorKind::AlreadyInstalled,
                current,
                requested,
            ));
        }
        self.preference_storage.replace_backend(backend);
        state.installed = true;
        Ok(())
    }

    pub fn preference_storage_backend_kind(&self) -> PreferenceStorageBackendKind {
        self.preference_storage.backend_kind()
    }

    pub(crate) fn preference_persistence_adapter(&self) -> Arc<PreferencePersistenceAdapter> {
        Arc::clone(&self.preference_storage)
    }

    fn lock_install_state(&self) -> MutexGuard<'_, PreferenceStorageBackendInstallState> {
        self.install_state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for PlatformDriver {
    fn default() -> Self {
        Self {
            preference_storage: Arc::new(PreferencePersistenceAdapter::new(
                Arc::new(UnavailablePreferenceStorageBackend),
                PreferencePersistenceLimits::default(),
            )),
            install_state: Mutex::new(PreferenceStorageBackendInstallState::default()),
        }
    }
}

impl fmt::Debug for PlatformDriver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PlatformDriver")
            .field(
                "preference_storage_backend_kind",
                &self.preference_storage_backend_kind(),
            )
            .finish()
    }
}

#[derive(Default)]
struct PreferenceStorageBackendInstallState {
    installed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferenceStorageBackendInstallErrorKind {
    UnavailableBackend,
    AlreadyInstalled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreferenceStorageBackendInstallError {
    kind: PreferenceStorageBackendInstallErrorKind,
    current_backend: PreferenceStorageBackendKind,
    requested_backend: PreferenceStorageBackendKind,
}

impl PreferenceStorageBackendInstallError {
    const fn new(
        kind: PreferenceStorageBackendInstallErrorKind,
        current_backend: PreferenceStorageBackendKind,
        requested_backend: PreferenceStorageBackendKind,
    ) -> Self {
        Self {
            kind,
            current_backend,
            requested_backend,
        }
    }

    pub const fn kind(self) -> PreferenceStorageBackendInstallErrorKind {
        self.kind
    }

    pub const fn current_backend(self) -> PreferenceStorageBackendKind {
        self.current_backend
    }

    pub const fn requested_backend(self) -> PreferenceStorageBackendKind {
        self.requested_backend
    }
}

impl fmt::Display for PreferenceStorageBackendInstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cannot install preference storage backend {} over {}: {:?}",
            self.requested_backend.as_str(),
            self.current_backend.as_str(),
            self.kind
        )
    }
}

impl Error for PreferenceStorageBackendInstallError {}
