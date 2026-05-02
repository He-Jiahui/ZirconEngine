use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataAssetFormat {
    Toml,
    Json,
    Yaml,
    Xml,
}

impl DataAssetFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Toml => "toml",
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Xml => "xml",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataAsset {
    pub uri: AssetUri,
    pub format: DataAssetFormat,
    pub text: String,
    #[serde(default)]
    pub canonical_json: serde_json::Value,
}
