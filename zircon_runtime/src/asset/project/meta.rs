use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::asset::{AssetKind, AssetUri, AssetUuid};

const ASSET_META_FORMAT_VERSION: u32 = 3;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_locator: Option<AssetUri>,
    #[serde(default)]
    pub importer_id: String,
    #[serde(default)]
    pub import_settings: toml::Table,
    #[serde(default)]
    pub config_hash: String,
    #[serde(default)]
    pub source_mtime_unix_ms: u64,
    #[serde(default)]
    pub source_hash: String,
    #[serde(default)]
    pub preview_state: PreviewState,
    #[serde(default)]
    pub importer_version: u32,
    #[serde(default)]
    pub source_schema_version: Option<u32>,
    #[serde(default)]
    pub target_schema_version: Option<u32>,
    #[serde(default)]
    pub migration_summary: String,
}

impl AssetMetaDocument {
    pub fn new(asset_uuid: AssetUuid, primary_locator: AssetUri, kind: AssetKind) -> Self {
        Self {
            format_version: ASSET_META_FORMAT_VERSION,
            asset_uuid,
            primary_locator,
            kind,
            artifact_locator: None,
            importer_id: String::new(),
            import_settings: toml::Table::new(),
            config_hash: String::new(),
            source_mtime_unix_ms: 0,
            source_hash: String::new(),
            preview_state: PreviewState::Dirty,
            importer_version: 0,
            source_schema_version: None,
            target_schema_version: None,
            migration_summary: String::new(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let document = fs::read_to_string(path)?;
        let mut meta: Self = toml::from_str(&document).map_err(invalid_data)?;
        meta.migrate_to_current().map_err(|error| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
        })?;
        Ok(meta)
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

impl AssetMetaDocument {
    fn migrate_to_current(&mut self) -> Result<(), String> {
        if self.format_version > ASSET_META_FORMAT_VERSION {
            return Err(format!(
                "asset meta format version {} is newer than supported {}",
                self.format_version, ASSET_META_FORMAT_VERSION
            ));
        }
        if self.format_version < ASSET_META_FORMAT_VERSION {
            self.format_version = ASSET_META_FORMAT_VERSION;
            if self.migration_summary.is_empty() {
                self.migration_summary =
                    "meta document migrated to artifact metadata v3".to_string();
            }
        }
        Ok(())
    }
}

fn invalid_data(error: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}
