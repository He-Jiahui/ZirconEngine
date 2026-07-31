use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use crate::core::commands::EditorCommandPaletteMru;
use crate::core::editor_operation::EditorOperationPath;

use super::{
    EditorKeymapOverrides, SettingDefinition, SettingSchema, SettingValue, SettingsKey,
    SettingsRegistry, SettingsScope, SettingsStore,
};

pub const EDITOR_DESIGN_TOKENS_KEY: &str = "editor.appearance.design_tokens";
pub const EDITOR_KEYMAP_OVERRIDES_KEY: &str = "editor.commands.keymap_overrides";
pub const EDITOR_COMMAND_PALETTE_MRU_KEY: &str = "editor.commands.palette_mru";
pub const VIEWPORT_TRANSLATE_STEP_KEY: &str = "editor.viewport.translate_step";
pub const VIEWPORT_ROTATE_STEP_DEGREES_KEY: &str = "editor.viewport.rotate_step_degrees";
pub const VIEWPORT_SCALE_STEP_KEY: &str = "editor.viewport.scale_step";

const MINIMUM_VIEWPORT_SNAP_STEP: f64 = 0.0001;
const MAXIMUM_VIEWPORT_SNAP_STEP: f64 = 1_000_000.0;

/// Registers the settings owned by Editor17 itself. Other feature plans add their
/// definitions through this registry rather than creating another persistence path.
pub fn settings_registry_with_defaults() -> SettingsRegistry {
    let mut registry = SettingsRegistry::default();
    registry
        .register(
            SettingDefinition::new(
                SettingsKey::parse(EDITOR_DESIGN_TOKENS_KEY)
                    .expect("the built-in design-token key is valid"),
                SettingsScope::User,
                SettingSchema::DesignTokens,
                SettingValue::DesignTokens(EditorDesignTokens::workbench_dark()),
                false,
                "Appearance/Workbench",
            )
            .expect("the built-in design-token definition is valid"),
        )
        .expect("the built-in design-token definition is unique");
    registry
        .register(
            SettingDefinition::new(
                SettingsKey::parse(EDITOR_KEYMAP_OVERRIDES_KEY)
                    .expect("the built-in keymap-overrides key is valid"),
                SettingsScope::User,
                SettingSchema::KeymapOverrides,
                SettingValue::KeymapOverrides(EditorKeymapOverrides::default()),
                false,
                "Editor/Keyboard Shortcuts",
            )
            .expect("the built-in keymap-overrides definition is valid"),
        )
        .expect("the built-in keymap-overrides definition is unique");
    registry
        .register(
            SettingDefinition::new(
                SettingsKey::parse(EDITOR_COMMAND_PALETTE_MRU_KEY)
                    .expect("the built-in command-palette MRU key is valid"),
                SettingsScope::Session,
                SettingSchema::CommandPaletteMru,
                SettingValue::CommandPaletteMru(EditorCommandPaletteMru::default()),
                false,
                "Editor/Command Palette",
            )
            .expect("the built-in command-palette MRU definition is valid"),
        )
        .expect("the built-in command-palette MRU definition is unique");
    register_viewport_snap_step(
        &mut registry,
        VIEWPORT_TRANSLATE_STEP_KEY,
        SettingValue::Float(1.0),
        "Viewport/Snapping",
    );
    register_viewport_snap_step(
        &mut registry,
        VIEWPORT_ROTATE_STEP_DEGREES_KEY,
        SettingValue::Float(15.0),
        "Viewport/Snapping",
    );
    register_viewport_snap_step(
        &mut registry,
        VIEWPORT_SCALE_STEP_KEY,
        SettingValue::Float(0.1),
        "Viewport/Snapping",
    );
    registry
}

fn register_viewport_snap_step(
    registry: &mut SettingsRegistry,
    key: &str,
    default: SettingValue,
    category_path: &str,
) {
    registry
        .register(
            SettingDefinition::new(
                SettingsKey::parse(key).expect("the built-in viewport snap key is valid"),
                SettingsScope::Project,
                SettingSchema::Float {
                    minimum: MINIMUM_VIEWPORT_SNAP_STEP,
                    maximum: MAXIMUM_VIEWPORT_SNAP_STEP,
                },
                default,
                false,
                category_path,
            )
            .expect("the built-in viewport snap definition is valid"),
        )
        .expect("the built-in viewport snap definition is unique");
}

/// Resolves the User-layer appearance setting for retained-host startup. A missing
/// or invalid file never reactivates the retired preferences format; it falls back
/// to the registered default in the new settings registry.
pub fn editor_design_tokens_at_startup() -> EditorDesignTokens {
    let registry = settings_registry_at_startup();
    let key = SettingsKey::parse(EDITOR_DESIGN_TOKENS_KEY)
        .expect("the built-in design-token key is valid");
    match registry
        .resolve(&key)
        .expect("the built-in design-token setting is registered")
    {
        SettingValue::DesignTokens(tokens) => tokens.clone(),
        _ => unreachable!("the built-in design-token setting has a design-token schema"),
    }
}

pub(crate) fn settings_registry_at_startup() -> SettingsRegistry {
    let mut registry = settings_registry_with_defaults();
    match SettingsStore::from_user_environment() {
        Ok(store) => {
            if let Err(error) = store.load_into(SettingsScope::User, &mut registry) {
                tracing::warn!(error = %error, "failed to load editor user settings; using defaults");
            }
        }
        Err(error) => {
            tracing::warn!(error = %error, "failed to resolve editor user settings root; using defaults");
        }
    }
    registry
}

/// Resolves the effective keymap delta after User/Project/Session precedence.
/// The keymap owns only pure merging; this registry remains its sole settings
/// authority.
pub fn editor_keymap_overrides(registry: &SettingsRegistry) -> &EditorKeymapOverrides {
    let key = SettingsKey::parse(EDITOR_KEYMAP_OVERRIDES_KEY)
        .expect("the built-in keymap-overrides key is valid");
    match registry
        .resolve(&key)
        .expect("the built-in keymap-overrides setting is registered")
    {
        SettingValue::KeymapOverrides(overrides) => overrides,
        _ => unreachable!("the built-in keymap-overrides setting has a keymap-overrides schema"),
    }
}

/// Resolves the transient MRU command ordering owned by the Session layer.
pub(crate) fn editor_command_palette_mru(registry: &SettingsRegistry) -> &EditorCommandPaletteMru {
    let key = SettingsKey::parse(EDITOR_COMMAND_PALETTE_MRU_KEY)
        .expect("the built-in command-palette MRU key is valid");
    match registry
        .resolve(&key)
        .expect("the built-in command-palette MRU setting is registered")
    {
        SettingValue::CommandPaletteMru(mru) => mru,
        _ => unreachable!("the built-in command-palette MRU setting has an MRU schema"),
    }
}

/// Records a command only after its palette dispatch succeeds. Session values never persist.
pub(crate) fn record_editor_command_palette_usage(
    registry: &mut SettingsRegistry,
    command: EditorOperationPath,
) {
    let mut mru = editor_command_palette_mru(registry).clone();
    if !mru.record(command) {
        return;
    }
    let key = SettingsKey::parse(EDITOR_COMMAND_PALETTE_MRU_KEY)
        .expect("the built-in command-palette MRU key is valid");
    registry
        .set(
            SettingsScope::Session,
            &key,
            SettingValue::CommandPaletteMru(mru),
        )
        .expect("the built-in command-palette MRU setting should accept Session updates");
}
