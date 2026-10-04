use super::*;

#[test]
fn host_text_preferences_project_from_editor_typography_tokens() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.typography.ui_family = "ui-family".to_string();
    tokens.typography.ui_strong_family = "ui-strong-family".to_string();
    tokens.typography.code_family = "code-family".to_string();
    tokens.typography.utility_tab_text_role = EditorUtilityTabTextRole::Code;
    tokens.typography.font_smoothing = EditorFontSmoothing::Subpixel;
    tokens.typography.body_weight = 420;
    tokens.typography.strong_weight = 650;
    tokens.typography.code_weight = 430;

    let preferences = project_host_text_preferences(&tokens);

    assert_eq!(preferences.ui_family, "ui-family");
    assert_eq!(preferences.ui_strong_family, "ui-strong-family");
    assert_eq!(preferences.code_family, "code-family");
    assert_eq!(
        preferences.utility_tab_text_role,
        HostUtilityTabTextRole::Code
    );
    assert_eq!(preferences.smoothing, HostTextSmoothing::Subpixel);
    assert_eq!(preferences.ui_weight, 420);
    assert_eq!(preferences.strong_weight, 650);
    assert_eq!(preferences.code_weight, 430);
}

#[test]
fn host_text_preferences_default_to_logical_families() {
    let preferences = HostTextPreferences::default();

    assert_eq!(preferences.ui_family, "Fira Sans");
    assert_eq!(preferences.ui_strong_family, "Fira Sans");
    assert_eq!(preferences.code_family, "Fira Mono");
    assert_eq!(
        preferences.utility_tab_text_role,
        HostUtilityTabTextRole::Ui
    );
    assert_eq!(preferences.smoothing, HostTextSmoothing::Grayscale);
}

#[test]
fn host_text_preferences_reject_invalid_font_weights() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.typography.body_weight = 0;
    tokens.typography.strong_weight = 1_001;
    tokens.typography.code_weight = u16::MAX;

    let preferences = project_host_text_preferences(&tokens);
    let defaults = EditorTypographyTokens::workbench_default();

    assert_eq!(preferences.ui_weight, defaults.body_weight);
    assert_eq!(preferences.strong_weight, defaults.strong_weight);
    assert_eq!(preferences.code_weight, defaults.code_weight);
}

#[test]
fn host_text_preferences_preserve_valid_variable_font_weights() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.typography.body_weight = 1;
    tokens.typography.strong_weight = 650;
    tokens.typography.code_weight = 1_000;

    let preferences = project_host_text_preferences(&tokens);

    assert_eq!(preferences.ui_weight, 1);
    assert_eq!(preferences.strong_weight, 650);
    assert_eq!(preferences.code_weight, 1_000);
}
