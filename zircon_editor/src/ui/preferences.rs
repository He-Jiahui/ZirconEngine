use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::design_tokens::{
    EditorControlTokens, EditorDensityTokens, EditorDesignTokens, EditorPaletteTokens,
    EditorStateRoleTokens, EditorTypographyTokens, EDITOR_WORKBENCH_TOKENS_ID,
};

pub(crate) const APPEARANCE_PREFERENCES_VERSION: u32 = 1;
pub(crate) const APPEARANCE_PREFERENCES_PATH_ENV: &str = "ZIRCON_EDITOR_APPEARANCE_PREFERENCES";

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EditorAppearancePreferences {
    design_tokens: EditorDesignTokens,
}

impl Default for EditorAppearancePreferences {
    fn default() -> Self {
        Self::workbench_default()
    }
}

impl EditorAppearancePreferences {
    pub(crate) fn workbench_default() -> Self {
        Self {
            design_tokens: EditorDesignTokens::workbench_dark(),
        }
    }

    pub(crate) fn from_design_tokens(design_tokens: EditorDesignTokens) -> Self {
        Self { design_tokens }
    }

    pub(crate) fn with_typography(mut self, typography: EditorTypographyTokens) -> Self {
        self.design_tokens.typography = typography;
        self
    }

    pub(crate) fn with_palette(mut self, palette: EditorPaletteTokens) -> Self {
        self.design_tokens.palette = palette;
        self
    }

    pub(crate) fn with_controls(mut self, controls: EditorControlTokens) -> Self {
        self.design_tokens.controls = controls;
        self
    }

    pub(crate) fn with_density(mut self, density: EditorDensityTokens) -> Self {
        self.design_tokens.density = density;
        self
    }

    pub(crate) fn with_state_roles(mut self, state_roles: EditorStateRoleTokens) -> Self {
        self.design_tokens.state_roles = state_roles;
        self
    }

    pub(crate) fn design_tokens(&self) -> &EditorDesignTokens {
        &self.design_tokens
    }

    pub(crate) fn to_persistence_document(&self) -> EditorAppearancePreferencesDocument {
        EditorAppearancePreferencesDocument {
            version: APPEARANCE_PREFERENCES_VERSION,
            active_profile: self.design_tokens.id.clone(),
            design_tokens: self.design_tokens.clone(),
        }
    }

    pub(crate) fn from_persistence_document(document: EditorAppearancePreferencesDocument) -> Self {
        if document.version == APPEARANCE_PREFERENCES_VERSION {
            Self::from_design_tokens(document.design_tokens)
        } else {
            Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct EditorAppearancePreferencesDocument {
    pub(crate) version: u32,
    pub(crate) active_profile: String,
    pub(crate) design_tokens: EditorDesignTokens,
}

impl Default for EditorAppearancePreferencesDocument {
    fn default() -> Self {
        Self {
            version: APPEARANCE_PREFERENCES_VERSION,
            active_profile: EDITOR_WORKBENCH_TOKENS_ID.to_string(),
            design_tokens: EditorDesignTokens::workbench_dark(),
        }
    }
}

pub(crate) struct EditorAppearancePreferenceStore;

impl EditorAppearancePreferenceStore {
    pub(crate) fn serialize_to_string(
        preferences: &EditorAppearancePreferences,
    ) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(&preferences.to_persistence_document())
    }

    pub(crate) fn load_from_str(
        source: &str,
    ) -> Result<EditorAppearancePreferences, toml::de::Error> {
        let document: EditorAppearancePreferencesDocument = toml::from_str(source)?;
        Ok(EditorAppearancePreferences::from_persistence_document(
            document,
        ))
    }

    pub(crate) fn save_to_path(
        path: impl AsRef<Path>,
        preferences: &EditorAppearancePreferences,
    ) -> io::Result<()> {
        let source = Self::serialize_to_string(preferences)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        if let Some(parent) = path
            .as_ref()
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, source)
    }

    pub(crate) fn load_from_path(
        path: impl AsRef<Path>,
    ) -> io::Result<EditorAppearancePreferences> {
        let source = fs::read_to_string(path)?;
        Self::load_from_str(&source)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}

pub(crate) fn default_editor_appearance_preferences() -> EditorAppearancePreferences {
    EditorAppearancePreferences::default()
}

pub(crate) fn editor_startup_appearance_preferences() -> EditorAppearancePreferences {
    let path = appearance_preferences_path_from_env_value(std::env::var_os(
        APPEARANCE_PREFERENCES_PATH_ENV,
    ));
    editor_appearance_preferences_from_optional_path(path.as_deref())
}

pub(crate) fn editor_appearance_preferences_from_optional_path(
    path: Option<&Path>,
) -> EditorAppearancePreferences {
    let Some(path) = path else {
        return default_editor_appearance_preferences();
    };
    EditorAppearancePreferenceStore::load_from_path(path).unwrap_or_else(|error| {
        tracing::warn!(
            path = %path.display(),
            error = %error,
            "falling back to default editor appearance preferences"
        );
        default_editor_appearance_preferences()
    })
}

fn appearance_preferences_path_from_env_value(value: Option<OsString>) -> Option<PathBuf> {
    value
        .filter(|value| !value.as_os_str().is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::design_tokens::{
        EditorFontSmoothing, EditorUtilityTabTextRole,
    };

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

        let preferences =
            EditorAppearancePreferences::default().with_typography(typography.clone());

        assert_eq!(preferences.design_tokens().typography, typography);
    }

    #[test]
    fn appearance_preferences_can_replace_palette_and_style_tokens_globally() {
        let mut palette = EditorPaletteTokens::workbench_dark();
        palette.accent =
            zircon_runtime_interface::ui::style::UiRgbaColor::from_u8(9, 180, 220, 255);
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
    }

    #[test]
    fn appearance_preferences_load_unsupported_versions_as_default_tokens() {
        let mut document = EditorAppearancePreferencesDocument::default();
        document.version = APPEARANCE_PREFERENCES_VERSION + 1;
        document.design_tokens.typography.ui_family = "unsupported-ui-family".to_string();
        let source =
            toml::to_string_pretty(&document).expect("test document should serialize to toml");

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
        let path = std::env::temp_dir().join(format!(
            "zircon-editor-appearance-preferences-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));

        EditorAppearancePreferenceStore::save_to_path(&path, &preferences)
            .expect("appearance preferences should save to temp path");
        let restored = EditorAppearancePreferenceStore::load_from_path(&path)
            .expect("appearance preferences should load from temp path");
        let _ = fs::remove_file(&path);

        assert_eq!(restored.design_tokens(), &tokens);
    }

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

    fn temp_appearance_preferences_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "zircon-editor-appearance-preferences-{}-{}-{}.toml",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ))
    }
}
