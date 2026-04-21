use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ui::surface::UiTextRenderMode;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontAsset {
    pub source: String,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub render_mode: Option<UiTextRenderMode>,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum FontAssetError {
    #[error("failed to parse font asset document: {0}")]
    Parse(String),
}

impl FontAsset {
    pub fn from_toml_str(document: &str) -> Result<Self, FontAssetError> {
        toml::from_str(document).map_err(|error| FontAssetError::Parse(error.to_string()))
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}
