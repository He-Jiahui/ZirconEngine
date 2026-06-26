use serde::{Deserialize, Serialize};
use thiserror::Error;

use zircon_runtime_interface::ui::surface::UiTextRenderMode;

pub type FontAssetResult<T> = std::result::Result<T, FontAssetError>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontAsset {
    pub source: String,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub render_mode: Option<UiTextRenderMode>,
}

#[derive(Debug, Error)]
pub enum FontAssetError {
    #[error("failed to parse font asset document: {0}")]
    Parse(#[source] toml::de::Error),
}

impl FontAsset {
    pub fn from_toml_str(document: &str) -> FontAssetResult<Self> {
        toml::from_str(document).map_err(FontAssetError::Parse)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}
