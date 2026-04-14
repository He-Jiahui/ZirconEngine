use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::{AssetKind, AssetUri, AssetUuid};

const ASSET_META_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewState {
    #[default]
    Dirty,
    Ready,
    Error,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetMetaDocument {
    pub format_version: u32,
    pub asset_uuid: AssetUuid,
    pub primary_locator: AssetUri,
    pub kind: AssetKind,
    #[serde(default)]
    pub import_settings: toml::Table,
    #[serde(default)]
    pub editor_adapter: Option<String>,
    #[serde(default)]
    pub source_mtime_unix_ms: u64,
    #[serde(default)]
    pub source_hash: String,
    #[serde(default)]
    pub preview_state: PreviewState,
}

impl AssetMetaDocument {
    pub fn new(asset_uuid: AssetUuid, primary_locator: AssetUri, kind: AssetKind) -> Self {
        Self {
            format_version: ASSET_META_FORMAT_VERSION,
            asset_uuid,
            primary_locator,
            kind,
            import_settings: toml::Table::new(),
            editor_adapter: None,
            source_mtime_unix_ms: 0,
            source_hash: String::new(),
            preview_state: PreviewState::Dirty,
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let document = fs::read_to_string(path)?;
        toml::from_str(&document).map_err(invalid_data)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let document = toml::to_string_pretty(self).map_err(invalid_data)?;
        fs::write(path, document)
    }
}

fn invalid_data(error: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}
