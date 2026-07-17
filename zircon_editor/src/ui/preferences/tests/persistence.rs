use std::fs;

use zircon_runtime_interface::ui::design_tokens::{
    EditorDesignTokens, EditorFontSmoothing, EditorTypographyTokens, EditorUtilityTabTextRole,
    EDITOR_WORKBENCH_TOKENS_ID,
};

use super::super::appearance::EditorAppearancePreferences;
use super::super::persistence::{
    EditorAppearancePreferenceStore, EditorAppearancePreferencesDocument,
    APPEARANCE_PREFERENCES_VERSION,
};
use super::support::temp_appearance_preferences_path;

#[test]
fn appearance_preferences_document_defaults_to_logical_font_families() {
    let document = EditorAppearancePreferencesDocument::default();

    assert_eq!(document.version, APPEARANCE_PREFERENCES_VERSION);
    assert_eq!(document.active_profile, EDITOR_WORKBENCH_TOKENS_ID);
    assert_eq!(
        document.design_tokens.typography.ui_family,
        EditorTypographyTokens::DEFAULT_UI_FAMILY
    );
    assert_eq!(
        document.design_tokens.typography.code_family,
        EditorTypographyTokens::DEFAULT_CODE_FAMILY
    );
    assert_eq!(
        document.design_tokens.typography.font_smoothing,
        EditorFontSmoothing::Grayscale
    );
    assert_eq!(
        document.design_tokens.typography.utility_tab_text_role,
        EditorUtilityTabTextRole::Ui
    );
}

#[test]
fn appearance_preferences_roundtrip_full_design_tokens_through_toml() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.id = "project.dark.custom".to_string();
    tokens.typography.ui_family = "ui-family".to_string();
    tokens.typography.ui_strong_family = "ui-strong-family".to_string();
    tokens.typography.code_family = "code-family".to_string();
    tokens.typography.utility_tab_text_role = EditorUtilityTabTextRole::Code;
    tokens.typography.font_smoothing = EditorFontSmoothing::Subpixel;
    tokens.palette.accent =
        zircon_runtime_interface::ui::style::UiRgbaColor::from_u8(9, 180, 220, 255);
    tokens.controls.control_radius = 3.0;
    tokens.density.row_height = 30.0;

    let preferences = EditorAppearancePreferences::from_design_tokens(tokens.clone());
    let source = EditorAppearancePreferenceStore::serialize_to_string(&preferences)
        .expect("appearance preferences should serialize to toml");
    let restored = EditorAppearancePreferenceStore::load_from_str(&source)
        .expect("appearance preferences should deserialize from toml");

    assert_eq!(restored.design_tokens(), &tokens);
    assert!(source.contains("active_profile = \"project.dark.custom\""));
    assert!(source.contains("ui_family = \"ui-family\""));
    assert!(source.contains("utility_tab_text_role = \"code\""));
    assert!(source.contains("font_smoothing = \"subpixel\""));
}

#[test]
fn appearance_preferences_load_utility_tab_text_role_from_toml() {
    let source = r#"
version = 1
active_profile = "project.utility-tabs"

[design_tokens]
id = "project.utility-tabs"

[design_tokens.typography]
ui_family = "system-ui"
ui_strong_family = "system-ui"
code_family = "monospace"
utility_tab_text_role = "code"
body_size = 10.0
caption_size = 8.5
title_size = 14.0
body_weight = 400
strong_weight = 600
code_weight = 400
line_height = 1.2
font_smoothing = "grayscale"
"#;

    let restored = EditorAppearancePreferenceStore::load_from_str(source)
        .expect("utility tab text role should deserialize from appearance preferences");

    assert_eq!(
        restored.design_tokens().typography.utility_tab_text_role,
        EditorUtilityTabTextRole::Code
    );
    assert!(
        (restored.design_tokens().typography.body_size
            - EditorTypographyTokens::WORKBENCH_BODY_SIZE)
            .abs()
            < 0.001
    );
    assert!(
        (restored.design_tokens().typography.caption_size
            - EditorTypographyTokens::WORKBENCH_CAPTION_SIZE)
            .abs()
            < 0.001
    );
    assert!(
        (restored.design_tokens().typography.title_size
            - EditorTypographyTokens::WORKBENCH_TITLE_SIZE)
            .abs()
            < 0.001
    );
}

#[test]
fn appearance_preferences_load_unsupported_versions_as_default_tokens() {
    let mut document = EditorAppearancePreferencesDocument::default();
    document.version = APPEARANCE_PREFERENCES_VERSION + 1;
    document.design_tokens.typography.ui_family = "unsupported-ui-family".to_string();
    let source = toml::to_string_pretty(&document).expect("test document should serialize to toml");

    let restored = EditorAppearancePreferenceStore::load_from_str(&source)
        .expect("unsupported version should parse before falling back");

    assert_eq!(
        restored.design_tokens().typography.ui_family,
        EditorTypographyTokens::DEFAULT_UI_FAMILY
    );
}

#[test]
fn appearance_preferences_store_loads_saved_toml_from_path() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.id = "project.light.custom".to_string();
    tokens.typography.ui_family = "light-ui-family".to_string();
    tokens.palette.accent =
        zircon_runtime_interface::ui::style::UiRgbaColor::from_u8(200, 90, 40, 255);
    let preferences = EditorAppearancePreferences::from_design_tokens(tokens.clone());
    let path = temp_appearance_preferences_path("store");

    EditorAppearancePreferenceStore::save_to_path(&path, &preferences)
        .expect("appearance preferences should save to temp path");
    let restored = EditorAppearancePreferenceStore::load_from_path(&path)
        .expect("appearance preferences should load from temp path");
    let _ = fs::remove_file(&path);

    assert_eq!(restored.design_tokens(), &tokens);
}
