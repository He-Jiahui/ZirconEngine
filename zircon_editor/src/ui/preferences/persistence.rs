use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::design_tokens::{EditorDesignTokens, EDITOR_WORKBENCH_TOKENS_ID};

use super::appearance::EditorAppearancePreferences;

pub(crate) const APPEARANCE_PREFERENCES_VERSION: u32 = 2;

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
