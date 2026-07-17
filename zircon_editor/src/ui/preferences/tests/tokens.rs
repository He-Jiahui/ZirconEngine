use zircon_runtime_interface::ui::design_tokens::{
    EditorControlTokens, EditorDensityTokens, EditorDesignTokens, EditorPaletteTokens,
    EditorStateRoleTokens, EditorTypographyTokens, EditorUtilityTabTextRole,
};

use super::super::appearance::EditorAppearancePreferences;

#[test]
fn appearance_preferences_default_to_logical_font_families() {
    let preferences = EditorAppearancePreferences::default();
    let typography = &preferences.design_tokens().typography;

    assert_eq!(
        typography.ui_family,
        EditorTypographyTokens::DEFAULT_UI_FAMILY
    );
    assert_eq!(
        typography.ui_strong_family,
        EditorTypographyTokens::DEFAULT_UI_FAMILY
    );
    assert_eq!(
        typography.code_family,
        EditorTypographyTokens::DEFAULT_CODE_FAMILY
    );
    assert_eq!(
        typography.utility_tab_text_role,
        EditorUtilityTabTextRole::Ui
    );
}

#[test]
fn appearance_preferences_can_replace_typography_globally() {
    let mut typography = EditorTypographyTokens::workbench_default();
    typography.ui_family = "ui-family".to_string();
    typography.ui_strong_family = "ui-strong-family".to_string();
    typography.code_family = "code-family".to_string();
    typography.utility_tab_text_role = EditorUtilityTabTextRole::Code;

    let preferences = EditorAppearancePreferences::default().with_typography(typography.clone());

    assert_eq!(preferences.design_tokens().typography, typography);
}

#[test]
fn appearance_preferences_can_replace_palette_and_style_tokens_globally() {
    let mut palette = EditorPaletteTokens::workbench_dark();
    palette.accent = zircon_runtime_interface::ui::style::UiRgbaColor::from_u8(9, 180, 220, 255);
    let mut controls = EditorControlTokens::workbench_dense();
    controls.control_radius = 3.0;
    let mut density = EditorDensityTokens::workbench_dense();
    density.gap_small = 3.0;
    let state_roles = EditorStateRoleTokens::workbench_dark();

    let preferences = EditorAppearancePreferences::default()
        .with_palette(palette.clone())
        .with_controls(controls)
        .with_density(density)
        .with_state_roles(state_roles.clone());

    assert_eq!(preferences.design_tokens().palette, palette);
    assert_eq!(preferences.design_tokens().controls, controls);
    assert_eq!(preferences.design_tokens().density, density);
    assert_eq!(preferences.design_tokens().state_roles, state_roles);
}

#[test]
fn appearance_preferences_can_replace_the_full_design_token_set() {
    let mut tokens = EditorDesignTokens::workbench_dark();
    tokens.typography.ui_family = "project-ui-family".to_string();

    let preferences = EditorAppearancePreferences::from_design_tokens(tokens.clone());

    assert_eq!(preferences.design_tokens(), &tokens);
}
