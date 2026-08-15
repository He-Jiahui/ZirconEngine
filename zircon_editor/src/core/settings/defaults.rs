use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use crate::core::commands::EditorCommandPaletteMru;

use super::{
    EditorKeymapOverrides, SettingDefinition, SettingSchema, SettingValue, SettingsKey,
    SettingsPresentation, SettingsRegistry, SettingsScope,
};

pub const EDITOR_DESIGN_TOKENS_KEY: &str = "editor.appearance.design_tokens";
pub const EDITOR_KEYMAP_OVERRIDES_KEY: &str = "editor.commands.keymap_overrides";
pub const EDITOR_COMMAND_PALETTE_MRU_KEY: &str = "editor.commands.palette_mru";
pub const EDITOR_LOCALE_KEY: &str = "editor.language.locale";
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
                presentation(
                    "settings.editor.appearance.design_tokens.label",
                    "settings.editor.appearance.design_tokens.description",
                    &[
                        "settings.category.appearance",
                        "settings.category.workbench",
                    ],
                ),
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
                presentation(
                    "settings.editor.commands.keymap_overrides.label",
                    "settings.editor.commands.keymap_overrides.description",
                    &[
                        "settings.category.editor",
                        "settings.category.keyboard_shortcuts",
                    ],
                ),
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
                presentation(
                    "settings.editor.commands.palette_mru.label",
                    "settings.editor.commands.palette_mru.description",
                    &[
                        "settings.category.editor",
                        "settings.category.command_palette",
                    ],
                ),
            )
            .expect("the built-in command-palette MRU definition is valid"),
        )
        .expect("the built-in command-palette MRU definition is unique");
    registry
        .register(
            SettingDefinition::new(
                SettingsKey::parse(EDITOR_LOCALE_KEY).expect("the built-in locale key is valid"),
                SettingsScope::User,
                SettingSchema::Enum {
                    variants: ["en".to_owned(), "zh-CN".to_owned()].into_iter().collect(),
                },
                SettingValue::Enum("en".to_owned()),
                false,
                presentation(
                    "settings.editor.language.locale.label",
                    "settings.editor.language.locale.description",
                    &["settings.category.editor", "settings.category.language"],
                ),
            )
            .expect("the built-in locale definition is valid"),
        )
        .expect("the built-in locale definition is unique");
    register_viewport_snap_step(
        &mut registry,
        VIEWPORT_TRANSLATE_STEP_KEY,
        SettingValue::Float(1.0),
        presentation(
            "settings.editor.viewport.translate_step.label",
            "settings.editor.viewport.translate_step.description",
            &["settings.category.viewport", "settings.category.snapping"],
        ),
    );
    register_viewport_snap_step(
        &mut registry,
        VIEWPORT_ROTATE_STEP_DEGREES_KEY,
        SettingValue::Float(15.0),
        presentation(
            "settings.editor.viewport.rotate_step_degrees.label",
            "settings.editor.viewport.rotate_step_degrees.description",
            &["settings.category.viewport", "settings.category.snapping"],
        ),
    );
    register_viewport_snap_step(
        &mut registry,
        VIEWPORT_SCALE_STEP_KEY,
        SettingValue::Float(0.1),
        presentation(
            "settings.editor.viewport.scale_step.label",
            "settings.editor.viewport.scale_step.description",
            &["settings.category.viewport", "settings.category.snapping"],
        ),
    );
    registry
}

fn register_viewport_snap_step(
    registry: &mut SettingsRegistry,
    key: &str,
    default: SettingValue,
    presentation: SettingsPresentation,
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
                presentation,
            )
            .expect("the built-in viewport snap definition is valid"),
        )
        .expect("the built-in viewport snap definition is unique");
}

fn presentation(
    label_key: &str,
    description_key: &str,
    category_path: &[&str],
) -> SettingsPresentation {
    SettingsPresentation::new(label_key, description_key, category_path.iter().copied())
        .expect("built-in settings presentation keys are valid")
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
