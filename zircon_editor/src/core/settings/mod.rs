mod defaults;
mod definition;
mod io;
mod keymap_overrides;
mod page;
mod registry;
mod scope;
#[cfg(test)]
mod tests;

pub(crate) use defaults::{
    editor_command_palette_mru, record_editor_command_palette_usage, settings_registry_at_startup,
};
pub use defaults::{
    editor_design_tokens_at_startup, editor_keymap_overrides, settings_registry_with_defaults,
    EDITOR_COMMAND_PALETTE_MRU_KEY, EDITOR_KEYMAP_OVERRIDES_KEY, VIEWPORT_ROTATE_STEP_DEGREES_KEY,
    VIEWPORT_SCALE_STEP_KEY, VIEWPORT_TRANSLATE_STEP_KEY,
};
pub use definition::{SettingDefinition, SettingSchema, SettingValue, SettingsKey};
pub use io::{
    SettingsDecodeError, SettingsLoad, SettingsPaths, SettingsStore, SettingsStoreError,
    SETTINGS_USER_ROOT_ENV,
};
pub use keymap_overrides::EditorKeymapOverrides;
pub use page::SettingsPageDescriptor;
pub use registry::{SettingChange, SettingsError, SettingsRegistry};
pub use scope::SettingsScope;
