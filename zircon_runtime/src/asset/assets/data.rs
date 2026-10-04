//! 文本类 importer 保留原文与可选的规范 JSON 投影；缓存与资源消费者可以分别使用保真文本和结构化视图，不应把 format 当作已完成语义校验的证明。

use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataAssetFormat {
    Text,
    Toml,
    Json,
    Yaml,
    Xml,
}

impl DataAssetFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
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
