#[path = "bundle.rs"]
mod bundle;
#[path = "catalog.rs"]
mod catalog;
#[path = "error.rs"]
mod error;
#[path = "locale.rs"]
mod locale;
#[path = "macros.rs"]
mod macros;
#[path = "service.rs"]
mod service;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub use bundle::{EditorLocalizationBundle, EditorLocalizationBundleId, EditorLocalizationKey};
pub use catalog::EditorI18nCatalog;
pub use error::EditorI18nError;
pub use locale::EditorLocale;
pub use service::{
    EditorI18nEventDiagnostics, EditorI18nEventSink, EditorI18nService, LocaleChangeDelivery,
};

#[cfg(test)]
use service::{MAX_PENDING_LOCALE_EVENTS, MAX_PENDING_LOCALE_EVENT_BYTES};
