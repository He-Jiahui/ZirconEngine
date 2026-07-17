use std::ffi::OsString;
use std::fs;

use zircon_runtime_interface::ui::design_tokens::EditorDesignTokens;

use super::super::appearance::EditorAppearancePreferences;
use super::super::persistence::EditorAppearancePreferenceStore;
use super::super::startup::{
    appearance_preferences_path_from_env_value, default_editor_appearance_preferences,
    editor_appearance_preferences_from_optional_path,
};
use super::support::temp_appearance_preferences_path;

#[test]
fn appearance_preferences_env_path_ignores_missing_or_empty_values() {
    assert_eq!(appearance_preferences_path_from_env_value(None), None);
    assert_eq!(
        appearance_preferences_path_from_env_value(Some(OsString::new())),
        None
    );
}

#[test]
fn appearance_preferences_startup_loads_saved_global_tokens_from_path() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.id = "project.startup.custom".to_string();
    tokens.typography.ui_family = "startup-ui-family".to_string();
    tokens.controls.default_height = 34.0;
    tokens.density.row_height = 31.0;
    let preferences = EditorAppearancePreferences::from_design_tokens(tokens.clone());
    let path = temp_appearance_preferences_path("startup-load");

    EditorAppearancePreferenceStore::save_to_path(&path, &preferences)
        .expect("appearance preferences should save for startup load");
    let loaded = editor_appearance_preferences_from_optional_path(Some(&path));
    let _ = fs::remove_file(&path);

    assert_eq!(loaded.design_tokens(), &tokens);
}

#[test]
fn appearance_preferences_startup_falls_back_for_missing_or_invalid_path() {
    let missing = temp_appearance_preferences_path("missing");
    assert_eq!(
        editor_appearance_preferences_from_optional_path(Some(&missing)).design_tokens(),
        default_editor_appearance_preferences().design_tokens()
    );

    let invalid = temp_appearance_preferences_path("invalid");
    fs::write(&invalid, "not = [valid")
        .expect("invalid preference fixture should write to temp path");
    let loaded = editor_appearance_preferences_from_optional_path(Some(&invalid));
    let _ = fs::remove_file(&invalid);

    assert_eq!(
        loaded.design_tokens(),
        default_editor_appearance_preferences().design_tokens()
    );
}
