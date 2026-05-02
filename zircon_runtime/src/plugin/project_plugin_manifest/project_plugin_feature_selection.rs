use serde::{Deserialize, Serialize};

use crate::{plugin::ExportPackagingStrategy, RuntimeTargetMode};

use super::default_packaging::default_packaging;
use super::default_true::default_true;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectPluginFeatureSelection {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub target_modes: Vec<RuntimeTargetMode>,
    #[serde(default = "default_packaging")]
    pub packaging: ExportPackagingStrategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_crate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor_crate: Option<String>,
}

impl ProjectPluginFeatureSelection {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            enabled: true,
            required: false,
            target_modes: Vec::new(),
            packaging: ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: None,
            editor_crate: None,
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_runtime_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.runtime_crate = Some(crate_name.into());
        self
    }

    pub fn with_editor_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.editor_crate = Some(crate_name.into());
        self
    }

    pub fn with_packaging(mut self, packaging: ExportPackagingStrategy) -> Self {
        self.packaging = packaging;
        self
    }

    pub fn with_target_modes(
        mut self,
        target_modes: impl IntoIterator<Item = RuntimeTargetMode>,
    ) -> Self {
        self.target_modes = target_modes.into_iter().collect();
        self
    }

    pub fn supports_target(&self, target: RuntimeTargetMode) -> bool {
        self.target_modes.is_empty() || self.target_modes.contains(&target)
    }

    pub fn runtime_crate_name(&self) -> String {
        self.runtime_crate
            .clone()
            .unwrap_or_else(|| format!("zircon_plugin_{}_runtime", feature_crate_stem(&self.id)))
    }

    pub fn runtime_crate_path(&self, owner_plugin_id: &str) -> String {
        format!(
            "{owner_plugin_id}/features/{}/runtime",
            feature_directory_slug(&self.id)
        )
    }
}

pub(super) fn feature_directory_slug(feature_id: &str) -> String {
    feature_id
        .rsplit_once('.')
        .map(|(_, suffix)| suffix)
        .unwrap_or(feature_id)
        .chars()
        .map(sanitize_crate_path_character)
        .collect()
}

fn feature_crate_stem(feature_id: &str) -> String {
    feature_id
        .chars()
        .map(sanitize_crate_path_character)
        .collect()
}

fn sanitize_crate_path_character(character: char) -> char {
    match character {
        'a'..='z' | '0'..='9' | '_' => character,
        'A'..='Z' => character.to_ascii_lowercase(),
        _ => '_',
    }
}
