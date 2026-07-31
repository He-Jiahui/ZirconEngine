use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::asset::{AssetKind, AssetUri, AssetUuid};

use crate::foundation::persistence::atomic_file::atomic_write;

const ASSET_META_FORMAT_VERSION: u32 = 7;

pub type AssetMetaResult<T> = std::result::Result<T, AssetMetaError>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum AssetMetaError {
    #[error("asset meta document is missing integer `format_version`")]
    MissingFormatVersion,
    #[error("asset meta `format_version` must be an integer")]
    NonIntegerFormatVersion,
    #[error("asset meta format version cannot be negative: {found}")]
    NegativeFormatVersion { found: i64 },
    #[error("asset meta format version is outside the supported u32 range: {found}")]
    OutOfRangeFormatVersion { found: i64 },
    #[error(
        "asset meta format version {found} is retired; minimum supported version is {minimum}"
    )]
    UnsupportedOldFormatVersion { found: u32, minimum: u32 },
    #[error("asset meta format version {found} is newer than supported {supported}")]
    UnsupportedFutureFormatVersion { found: u32, supported: u32 },
    #[error("asset meta field `source_hash` is retired; write `source_digest`")]
    RetiredSourceHashField,
    #[error("asset meta {scope} tag cannot be empty")]
    EmptyTag { scope: String },
    #[error("asset meta {scope} tag `{tag}` has leading or trailing whitespace")]
    TagHasSurroundingWhitespace { scope: String, tag: String },
    #[error("asset meta {scope} tag `{tag}` contains a control character")]
    TagContainsControlCharacter { scope: String, tag: String },
    #[error("asset meta {scope} contains duplicate tag `{tag}`")]
    DuplicateTag { scope: String, tag: String },
    #[error("failed to deserialize asset meta document: {message}")]
    DeserializeDocument { message: String },
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

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
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
    pub source_digest: String,
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
    pub tags: BTreeSet<String>,
    #[serde(default)]
    pub entries: Vec<AssetMetaEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetMetaEntry {
    pub uuid: AssetUuid,
    pub url: AssetUri,
    pub asset_kind: AssetKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_locator: Option<AssetUri>,
    #[serde(default)]
    pub dependencies: Vec<AssetUri>,
    #[serde(default)]
    pub tags: BTreeSet<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAssetMetaDocument {
    format_version: u32,
    uuid: AssetUuid,
    url: AssetUri,
    asset_kind: AssetKind,
    #[serde(default)]
    unit: AssetSourceUnit,
    #[serde(default)]
    included_files: Vec<AssetUri>,
    #[serde(default)]
    artifact_locator: Option<AssetUri>,
    #[serde(default)]
    importer_id: String,
    #[serde(default)]
    import_settings: toml::Table,
    #[serde(default)]
    config_hash: String,
    #[serde(default)]
    source_mtime_unix_ms: u64,
    #[serde(default)]
    source_digest: String,
    #[serde(default)]
    preview_state: PreviewState,
    #[serde(default)]
    importer_version: u32,
    #[serde(default)]
    source_schema_version: Option<u32>,
    #[serde(default)]
    target_schema_version: Option<u32>,
    #[serde(default)]
    migration_summary: String,
    #[serde(default)]
    dependencies: Vec<AssetUri>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    entries: Vec<RawAssetMetaEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAssetMetaEntry {
    uuid: AssetUuid,
    url: AssetUri,
    asset_kind: AssetKind,
    #[serde(default)]
    artifact_locator: Option<AssetUri>,
    #[serde(default)]
    dependencies: Vec<AssetUri>,
    #[serde(default)]
    tags: Vec<String>,
}

impl<'de> Deserialize<'de> for AssetMetaDocument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawAssetMetaDocument::deserialize(deserializer)?;
        validate_format_version(raw.format_version).map_err(D::Error::custom)?;
        validate_tag_list("root", &raw.tags).map_err(D::Error::custom)?;
        for (index, entry) in raw.entries.iter().enumerate() {
            validate_tag_list(&format!("entries[{index}]"), &entry.tags)
                .map_err(D::Error::custom)?;
        }
        Ok(raw.into_document())
    }
}

impl<'de> Deserialize<'de> for AssetMetaEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawAssetMetaEntry::deserialize(deserializer)?;
        validate_tag_list("entry", &raw.tags).map_err(D::Error::custom)?;
        Ok(raw.into_entry())
    }
}

impl RawAssetMetaDocument {
    fn into_document(self) -> AssetMetaDocument {
        AssetMetaDocument {
            format_version: self.format_version,
            uuid: self.uuid,
            url: self.url,
            asset_kind: self.asset_kind,
            unit: self.unit,
            included_files: self.included_files,
            artifact_locator: self.artifact_locator,
            importer_id: self.importer_id,
            import_settings: self.import_settings,
            config_hash: self.config_hash,
            source_mtime_unix_ms: self.source_mtime_unix_ms,
            source_digest: self.source_digest,
            preview_state: self.preview_state,
            importer_version: self.importer_version,
            source_schema_version: self.source_schema_version,
            target_schema_version: self.target_schema_version,
            migration_summary: self.migration_summary,
            dependencies: self.dependencies,
            tags: self.tags.into_iter().collect(),
            entries: self
                .entries
                .into_iter()
                .map(RawAssetMetaEntry::into_entry)
                .collect(),
        }
    }
}

impl RawAssetMetaEntry {
    fn into_entry(self) -> AssetMetaEntry {
        AssetMetaEntry {
            uuid: self.uuid,
            url: self.url,
            asset_kind: self.asset_kind,
            artifact_locator: self.artifact_locator,
            dependencies: self.dependencies,
            tags: self.tags.into_iter().collect(),
        }
    }
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
            source_digest: String::new(),
            preview_state: PreviewState::Dirty,
            importer_version: 0,
            source_schema_version: None,
            target_schema_version: None,
            migration_summary: String::new(),
            dependencies: Vec::new(),
            tags: BTreeSet::new(),
            entries: Vec::new(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let document = fs::read_to_string(path)?;
        Self::from_toml_str(&document).map_err(invalid_data)
    }

    pub fn from_toml_str(document: &str) -> AssetMetaResult<Self> {
        let value: toml::Value = toml::from_str(document).map_err(deserialize_error)?;
        let table = value
            .as_table()
            .ok_or_else(|| AssetMetaError::DeserializeDocument {
                message: "root value must be a table".to_string(),
            })?;
        let version = table
            .get("format_version")
            .ok_or(AssetMetaError::MissingFormatVersion)?;
        let version = version
            .as_integer()
            .ok_or(AssetMetaError::NonIntegerFormatVersion)?;
        if version < 0 {
            return Err(AssetMetaError::NegativeFormatVersion { found: version });
        }
        let found = u32::try_from(version)
            .map_err(|_| AssetMetaError::OutOfRangeFormatVersion { found: version })?;
        validate_format_version(found)?;
        if table.contains_key("source_hash") {
            return Err(AssetMetaError::RetiredSourceHashField);
        }
        validate_serialized_tags(table)?;
        let meta: Self = value.try_into().map_err(deserialize_error)?;
        Ok(meta)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let document = self.to_pretty_bytes()?;
        atomic_write(path.as_ref(), &document)
    }

    pub(crate) fn to_pretty_bytes(&self) -> Result<Vec<u8>, std::io::Error> {
        self.validate_current().map_err(invalid_data)?;
        let document = toml::to_string_pretty(self).map_err(invalid_data)?;
        Ok(document.into_bytes())
    }
}

impl AssetMetaDocument {
    fn validate_current(&self) -> AssetMetaResult<()> {
        validate_format_version(self.format_version)?;
        validate_tag_set("root", &self.tags)?;
        for (index, entry) in self.entries.iter().enumerate() {
            validate_tag_set(&format!("entries[{index}]"), &entry.tags)?;
        }
        Ok(())
    }
}

fn validate_serialized_tags(table: &toml::Table) -> AssetMetaResult<()> {
    validate_tag_value("root", table.get("tags"))?;
    if let Some(entries) = table.get("entries").and_then(toml::Value::as_array) {
        for (index, entry) in entries.iter().enumerate() {
            if let Some(entry) = entry.as_table() {
                validate_tag_value(&format!("entries[{index}]"), entry.get("tags"))?;
            }
        }
    }
    Ok(())
}

fn validate_tag_value(scope: &str, value: Option<&toml::Value>) -> AssetMetaResult<()> {
    let Some(tags) = value.and_then(toml::Value::as_array) else {
        return Ok(());
    };
    let mut seen = HashSet::new();
    for tag in tags.iter().filter_map(toml::Value::as_str) {
        validate_tag(scope, tag)?;
        if !seen.insert(tag) {
            return Err(AssetMetaError::DuplicateTag {
                scope: scope.to_string(),
                tag: tag.to_string(),
            });
        }
    }
    Ok(())
}

fn validate_tag_list(scope: &str, tags: &[String]) -> AssetMetaResult<()> {
    let mut seen = HashSet::new();
    for tag in tags {
        validate_tag(scope, tag)?;
        if !seen.insert(tag) {
            return Err(AssetMetaError::DuplicateTag {
                scope: scope.to_string(),
                tag: tag.clone(),
            });
        }
    }
    Ok(())
}

fn validate_tag_set(scope: &str, tags: &BTreeSet<String>) -> AssetMetaResult<()> {
    for tag in tags {
        validate_tag(scope, tag)?;
    }
    Ok(())
}

fn validate_tag(scope: &str, tag: &str) -> AssetMetaResult<()> {
    if tag.is_empty() {
        return Err(AssetMetaError::EmptyTag {
            scope: scope.to_string(),
        });
    }
    if tag.trim() != tag {
        return Err(AssetMetaError::TagHasSurroundingWhitespace {
            scope: scope.to_string(),
            tag: tag.to_string(),
        });
    }
    if tag.chars().any(char::is_control) {
        return Err(AssetMetaError::TagContainsControlCharacter {
            scope: scope.to_string(),
            tag: tag.to_string(),
        });
    }
    Ok(())
}

fn validate_format_version(found: u32) -> AssetMetaResult<()> {
    if found < ASSET_META_FORMAT_VERSION {
        return Err(AssetMetaError::UnsupportedOldFormatVersion {
            found,
            minimum: ASSET_META_FORMAT_VERSION,
        });
    }
    if found > ASSET_META_FORMAT_VERSION {
        return Err(AssetMetaError::UnsupportedFutureFormatVersion {
            found,
            supported: ASSET_META_FORMAT_VERSION,
        });
    }
    Ok(())
}

fn deserialize_error(error: toml::de::Error) -> AssetMetaError {
    AssetMetaError::DeserializeDocument {
        message: error.to_string(),
    }
}

fn invalid_data(error: impl std::error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_meta_validation_reports_typed_future_version_error() {
        let mut meta = AssetMetaDocument::new(
            AssetUuid::new(),
            AssetUri::parse("res://data/future.json").unwrap(),
            AssetKind::Data,
        );
        meta.format_version = ASSET_META_FORMAT_VERSION + 1;

        let error = meta
            .validate_current()
            .expect_err("future meta version should fail");

        assert_eq!(
            error,
            AssetMetaError::UnsupportedFutureFormatVersion {
                found: ASSET_META_FORMAT_VERSION + 1,
                supported: ASSET_META_FORMAT_VERSION,
            }
        );
    }
}
