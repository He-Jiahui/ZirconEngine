mod adapter;
mod overlay;
mod work;

pub use adapter::{
    PreferencePersistenceAdapter, PreferencePersistenceLimits, PreferencePersistenceQuote,
    MAX_PREFERENCE_FAILURE_DETAIL_BYTES, MAX_PREFERENCE_VALUE_BYTES,
};
pub use work::PreferenceBackendWorkAuthority;

#[cfg(test)]
mod tests;
