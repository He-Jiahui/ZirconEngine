use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::asset::{AssetKind, AssetUri, AssetUuid};

const ASSET_META_FORMAT_VERSION: u32 = 6;

pub type AssetMetaResult<T> = std::result::Result<T, AssetMetaError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AssetMetaError {
    #[error("asset meta format version {found} is newer than supported {supported}")]
    UnsupportedFormatVersion { found: u32, supported: u32 },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetSourceUnit {
    #[default]
    Single,
    Compound,
}

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
    pub uuid: AssetUuid,
    pub url: AssetUri,
    pub asset_kind: AssetKind,
    #[serde(default)]
    pub unit: AssetSourceUnit,
    #[serde(default)]
    pub included_files: Vec<AssetUri>,
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
    #[serde(default)]
    pub dependencies: Vec<AssetUri>,
    #[serde(default)]
    pub entries: Vec<AssetMetaEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssetMetaEntry {
    pub uuid: AssetUuid,
    pub url: AssetUri,
    pub asset_kind: AssetKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_locator: Option<AssetUri>,
    #[serde(default)]
    pub dependencies: Vec<AssetUri>,
}

impl AssetMetaDocument {
    pub fn new(uuid: AssetUuid, url: AssetUri, asset_kind: AssetKind) -> Self {
        Self {
            format_version: ASSET_META_FORMAT_VERSION,
            uuid,
            url,
            asset_kind,
            unit: AssetSourceUnit::Single,
            included_files: Vec::new(),
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
            dependencies: Vec::new(),
            entries: Vec::new(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let document = fs::read_to_string(path)?;
        let mut meta: Self = toml::from_str(&document).map_err(invalid_data)?;
        meta.migrate_to_current().map_err(invalid_data)?;
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
    fn migrate_to_current(&mut self) -> AssetMetaResult<()> {
        if self.format_version > ASSET_META_FORMAT_VERSION {
            return Err(AssetMetaError::UnsupportedFormatVersion {
                found: self.format_version,
                supported: ASSET_META_FORMAT_VERSION,
            });
        }
        if self.format_version < ASSET_META_FORMAT_VERSION {
            self.format_version = ASSET_META_FORMAT_VERSION;
            if self.migration_summary.is_empty() {
                self.migration_summary =
                    "meta document migrated to zmeta asset identity metadata v6".to_string();
            }
        }
        Ok(())
    }
}

fn invalid_data(error: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_meta_migration_reports_typed_future_version_error() {
        let mut meta = AssetMetaDocument::new(
            AssetUuid::new(),
            AssetUri::parse("res://data/future.json").unwrap(),
            AssetKind::Data,
        );
        meta.format_version = ASSET_META_FORMAT_VERSION + 1;

        let error = meta
            .migrate_to_current()
            .expect_err("future meta version should fail");

        assert_eq!(
            error,
            AssetMetaError::UnsupportedFormatVersion {
                found: ASSET_META_FORMAT_VERSION + 1,
                supported: ASSET_META_FORMAT_VERSION,
            }
        );
    }
}
