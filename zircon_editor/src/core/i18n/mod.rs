mod catalog;
mod error;
mod locale;
mod macros;
mod service;

#[cfg(test)]
mod tests;

pub use catalog::EditorI18nCatalog;
pub use error::EditorI18nError;
pub use locale::EditorLocale;
pub use service::{
    EditorI18nEventDiagnostics, EditorI18nEventSink, EditorI18nService, LocaleChangeDelivery,
};

#[cfg(test)]
use service::{MAX_PENDING_LOCALE_EVENTS, MAX_PENDING_LOCALE_EVENT_BYTES};
