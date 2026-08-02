mod adapter;
mod overlay;
mod work;

pub(crate) use adapter::{
    PreferencePersistenceAdapter, PreferencePersistenceDiagnostics, PreferencePersistenceLimits,
    PreferencePersistenceLimitsError, PreferencePersistenceQuote,
    MAX_PREFERENCE_FAILURE_DETAIL_BYTES, MAX_PREFERENCE_VALUE_BYTES,
};
pub(crate) use overlay::PreferenceOverlayDiagnostics;
pub use work::PreferenceBackendWorkAuthority;

#[cfg(test)]
mod tests;
