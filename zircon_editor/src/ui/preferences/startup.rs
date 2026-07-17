use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::appearance::EditorAppearancePreferences;
use super::persistence::EditorAppearancePreferenceStore;

pub(crate) const APPEARANCE_PREFERENCES_PATH_ENV: &str = "ZIRCON_EDITOR_APPEARANCE_PREFERENCES";

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

pub(super) fn appearance_preferences_path_from_env_value(
    value: Option<OsString>,
) -> Option<PathBuf> {
    value
        .filter(|value| !value.as_os_str().is_empty())
        .map(PathBuf::from)
}
