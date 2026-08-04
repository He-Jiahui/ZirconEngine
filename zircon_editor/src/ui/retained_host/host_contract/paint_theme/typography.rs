use std::sync::{OnceLock, RwLock};

use zircon_runtime_interface::ui::design_tokens::{
    EditorDesignTokens, EditorFontSmoothing, EditorTypographyTokens, EditorUtilityTabTextRole,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum HostTextSmoothing {
    Grayscale,
    Subpixel,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum HostUtilityTabTextRole {
    Ui,
    Code,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct HostTextPreferences {
    pub ui_family: String,
    pub ui_strong_family: String,
    pub code_family: String,
    pub utility_tab_text_role: HostUtilityTabTextRole,
    pub smoothing: HostTextSmoothing,
    pub ui_weight: u16,
    pub strong_weight: u16,
    pub code_weight: u16,
}

impl Default for HostTextPreferences {
    fn default() -> Self {
        project_host_text_preferences(&EditorDesignTokens::workbench_dark())
    }
}

impl HostTextPreferences {
    pub(in crate::ui::retained_host::host_contract) fn utility_tab_uses_code_text(&self) -> bool {
        self.utility_tab_text_role == HostUtilityTabTextRole::Code
    }
}

pub(crate) fn current_host_text_preferences() -> HostTextPreferences {
    match host_text_preferences().read() {
        Ok(preferences) => preferences.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

pub(crate) fn apply_host_text_preferences(preferences: HostTextPreferences) {
    match host_text_preferences().write() {
        Ok(mut current_preferences) => *current_preferences = preferences,
        Err(poisoned) => *poisoned.into_inner() = preferences,
    }
}

pub(crate) fn project_host_text_preferences(tokens: &EditorDesignTokens) -> HostTextPreferences {
    project_typography_tokens(&tokens.typography)
}

fn host_text_preferences() -> &'static RwLock<HostTextPreferences> {
    static PREFERENCES: OnceLock<RwLock<HostTextPreferences>> = OnceLock::new();
    PREFERENCES.get_or_init(|| RwLock::new(HostTextPreferences::default()))
}

fn project_typography_tokens(tokens: &EditorTypographyTokens) -> HostTextPreferences {
    let defaults = EditorTypographyTokens::workbench_default();
    HostTextPreferences {
        ui_family: normalized_family_or_default(
            tokens.ui_family.as_str(),
            EditorTypographyTokens::DEFAULT_UI_FAMILY,
        ),
        ui_strong_family: normalized_family_or_default(
            tokens.ui_strong_family.as_str(),
            tokens.ui_family.as_str(),
        ),
        code_family: normalized_family_or_default(
            tokens.code_family.as_str(),
            EditorTypographyTokens::DEFAULT_CODE_FAMILY,
        ),
        utility_tab_text_role: host_utility_tab_text_role_for_tokens(tokens.utility_tab_text_role),
        smoothing: host_text_smoothing_for_tokens(tokens.font_smoothing),
        ui_weight: valid_font_weight_or(tokens.body_weight, defaults.body_weight),
        strong_weight: valid_font_weight_or(tokens.strong_weight, defaults.strong_weight),
        code_weight: valid_font_weight_or(tokens.code_weight, defaults.code_weight),
    }
}

fn valid_font_weight_or(value: u16, fallback: u16) -> u16 {
    (1..=1000)
        .contains(&value)
        .then_some(value)
        .unwrap_or(fallback)
}

fn host_text_smoothing_for_tokens(smoothing: EditorFontSmoothing) -> HostTextSmoothing {
    match smoothing {
        EditorFontSmoothing::Grayscale => HostTextSmoothing::Grayscale,
        EditorFontSmoothing::Subpixel => HostTextSmoothing::Subpixel,
    }
}

fn host_utility_tab_text_role_for_tokens(role: EditorUtilityTabTextRole) -> HostUtilityTabTextRole {
    match role {
        EditorUtilityTabTextRole::Ui => HostUtilityTabTextRole::Ui,
        EditorUtilityTabTextRole::Code => HostUtilityTabTextRole::Code,
    }
}

fn normalized_family_or_default(value: &str, fallback: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        fallback.trim().to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
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

        assert_eq!(preferences.ui_family, "system-ui");
        assert_eq!(preferences.ui_strong_family, "system-ui");
        assert_eq!(preferences.code_family, "monospace");
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
}
