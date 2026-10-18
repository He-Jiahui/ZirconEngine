#[path = "authority.rs"]
mod authority;
#[path = "catalog/mod.rs"]
mod catalog;
#[path = "change_log.rs"]
mod change_log;
#[path = "defaults.rs"]
mod defaults;
#[path = "definition.rs"]
mod definition;
#[path = "io.rs"]
mod io;
#[path = "keymap_overrides.rs"]
mod keymap_overrides;
#[path = "mutation.rs"]
mod mutation;
#[path = "page.rs"]
mod page;
#[path = "persistence.rs"]
mod persistence;
#[path = "registry.rs"]
mod registry;
#[path = "scope.rs"]
mod scope;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "startup.rs"]
mod startup;
#[cfg(test)]
#[path = "tests/mod.rs"]
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
    EDITOR_LOCALE_KEY, EDITOR_SHOW_PREVIEW_FEATURES_KEY,
    VIEWPORT_ROTATE_SNAP_ENABLED_KEY, VIEWPORT_ROTATE_STEP_DEGREES_KEY,
    VIEWPORT_SCALE_SNAP_ENABLED_KEY, VIEWPORT_SCALE_STEP_KEY, VIEWPORT_TRANSLATE_SNAP_ENABLED_KEY,
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
    SettingsPersistenceShutdownError, SettingsPersistenceSubmitError, SettingsPersistenceTerminal,
    SettingsPersistenceTicket, SettingsPersistenceWaitResult,
};
pub use registry::{SettingsError, SettingsRegistry};
pub use scope::SettingsScope;
pub use snapshot::{SettingsSnapshot, ViewportSnapSettings};
pub(crate) use startup::SettingsStartup;
pub use startup::SettingsUserLayerLoad;
