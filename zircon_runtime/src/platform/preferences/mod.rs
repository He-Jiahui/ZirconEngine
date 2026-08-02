mod atomic_file;
mod backend;
mod persistence;
mod unavailable;

pub use atomic_file::AtomicFilePreferenceStorageBackend;
pub use backend::PreferenceStorageBackend;
pub use backend::PreferenceStorageBackendDiagnostics;
pub use persistence::PreferenceBackendWorkAuthority;
pub(crate) use persistence::{
    PreferenceOverlayDiagnostics, PreferencePersistenceAdapter, PreferencePersistenceDiagnostics,
    PreferencePersistenceLimits, PreferencePersistenceLimitsError, PreferencePersistenceQuote,
    MAX_PREFERENCE_FAILURE_DETAIL_BYTES, MAX_PREFERENCE_VALUE_BYTES,
};
pub(crate) use unavailable::UnavailablePreferenceStorageBackend;
pub(crate) type InstalledPreferenceBackendKind =
    crate::core::framework::platform::PreferenceStorageBackendKind;
