mod change_log;
mod defaults;
mod definition;
mod io;
mod keymap_overrides;
mod page;
mod persistence;
mod registry;
mod scope;
#[cfg(test)]
mod tests;

pub use crate::core::commands::EditorCommandPaletteMru;
pub use change_log::{
    SettingChange, SettingsChangeCursor, SettingsChangeDelta, SettingsChangeLogPolicy,
};
pub use defaults::{
    EDITOR_COMMAND_PALETTE_MRU_KEY, EDITOR_KEYMAP_OVERRIDES_KEY, EDITOR_LOCALE_KEY,
    VIEWPORT_ROTATE_STEP_DEGREES_KEY, VIEWPORT_SCALE_STEP_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
    editor_keymap_overrides, settings_registry_with_defaults,
};
pub use definition::{SettingDefinition, SettingSchema, SettingValue, SettingsKey};
pub use io::{
    SETTINGS_USER_ROOT_ENV, SettingsDecodeError, SettingsLoad, SettingsPaths, SettingsStore,
    SettingsStoreError,
};
pub use keymap_overrides::EditorKeymapOverrides;
pub use page::SettingsPageDescriptor;
pub use persistence::{
    SettingsPersistenceLimits, SettingsPersistenceRequest, SettingsPersistenceRetryError,
    SettingsPersistenceService, SettingsPersistenceShutdown, SettingsPersistenceShutdownError,
    SettingsPersistenceSubmitError, SettingsPersistenceTicket,
};
pub(crate) use registry::SettingsChangeSubscriber;
pub use registry::{
    SettingsAuthority, SettingsError, SettingsProjectLayerLoad, SettingsRegistry, SettingsSnapshot,
    ViewportSnapSettings,
};
pub use scope::SettingsScope;
