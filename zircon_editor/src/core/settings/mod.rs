mod authority;
mod catalog;
mod change_log;
mod defaults;
mod definition;
mod io;
mod keymap_overrides;
mod mutation;
mod page;
mod persistence;
mod registry;
mod scope;
mod snapshot;
mod startup;
#[cfg(test)]
mod tests;

pub use crate::core::commands::EditorCommandPaletteMru;
pub(crate) use authority::SettingsChangeSubscriber;
pub use authority::{SettingsAuthority, SettingsProjectLayerLoad};
pub use catalog::{
    ResolvedSettingSnapshot, ResolvedSettingValue, ResolvedSettingsBatch, SettingValueSource,
    SettingsCatalog,
};
pub use change_log::{
    SettingChange, SettingsChangeCursor, SettingsChangeDelta, SettingsChangeLogPolicy,
};
pub use defaults::{
    editor_keymap_overrides, settings_registry_with_defaults, EDITOR_AUTOSAVE_INTERVAL_SECS_KEY,
    EDITOR_COMMAND_PALETTE_MRU_KEY, EDITOR_DESIGN_TOKENS_KEY, EDITOR_KEYMAP_OVERRIDES_KEY,
    EDITOR_LOCALE_KEY, VIEWPORT_ROTATE_STEP_DEGREES_KEY, VIEWPORT_SCALE_STEP_KEY,
    VIEWPORT_TRANSLATE_STEP_KEY,
};
pub(crate) use definition::{SettingColorChannel, SettingNumericStepDirection};
pub use definition::{
    SettingDefinition, SettingSchema, SettingValue, SettingsKey, SettingsPresentation,
};
pub use io::{
    SettingsDecodeError, SettingsLoad, SettingsPaths, SettingsStore, SettingsStoreError,
    SETTINGS_USER_ROOT_ENV,
};
pub use keymap_overrides::EditorKeymapOverrides;
pub(crate) use mutation::SettingsPersistenceHealthSubscriber;
pub use mutation::{
    SettingsMutationCoordinator, SettingsMutationDisposition, SettingsMutationError,
    SettingsMutationReceipt, SettingsPersistenceDocumentHealth, SettingsPersistenceHealthSnapshot,
    SettingsPersistenceHealthStatus, SettingsPersistenceRetryDisposition,
    SettingsPersistenceRetryReceipt, SettingsProjectBindingReceipt,
};
pub use page::SettingsPageDescriptor;
pub use persistence::{
    SettingsFileGeneration, SettingsPersistenceLimits, SettingsPersistenceRequest,
    SettingsPersistenceRetryError, SettingsPersistenceService, SettingsPersistenceShutdown,
    SettingsPersistenceShutdownError, SettingsPersistenceSubmitError, SettingsPersistenceTicket,
};
pub use registry::{SettingsError, SettingsRegistry};
pub use scope::SettingsScope;
pub use snapshot::{SettingsSnapshot, ViewportSnapSettings};
pub(crate) use startup::SettingsStartup;
pub use startup::SettingsUserLayerLoad;
