use thiserror::Error;

use super::AssetImporterRegistryError;
use crate::asset::assets::{UiAssetDocumentError, UiV2AssetDocumentError};
use crate::core::resource::{ResourceLocator, ResourceLocatorError};

#[derive(Debug, Error)]
pub enum AssetImportError {
    #[error("asset I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("asset uri error: {0}")]
    Uri(#[from] ResourceLocatorError),
    #[error("asset parse failed: {0}")]
    Parse(String),
    #[error("unsupported asset format: {0}")]
    UnsupportedFormat(String),
    #[error("wgsl validation failed: {0}")]
    ShaderValidation(String),
    #[error("asset schema migration failed: {0}")]
    SchemaMigration(String),
    #[error("native asset importer failed: {0}")]
    Native(String),
    #[error("duplicate asset label {label} for source {source_uri}")]
    DuplicateAssetLabel {
        source_uri: ResourceLocator,
        label: String,
    },
    #[error("missing asset label {label} for source {source_uri}")]
    MissingAssetLabel {
        source_uri: ResourceLocator,
        label: String,
    },
    #[error("asset importer registry failed: {0}")]
    Registry(#[from] AssetImporterRegistryError),
    #[error("asset TOML serialization failed while {context}: {source}")]
    TomlSerialize {
        context: &'static str,
        #[source]
        source: toml::ser::Error,
    },
    #[error("asset TOML deserialization failed while {context}: {source}")]
    TomlDeserialize {
        context: &'static str,
        #[source]
        source: toml::de::Error,
    },
    #[error("cached TOML datetime `{value}` is invalid: {source}")]
    CachedTomlDatetime {
        value: String,
        #[source]
        source: toml::value::DatetimeParseError,
    },
    #[error("cached UI asset document failed while {context}: {source}")]
    UiDocument {
        context: &'static str,
        #[source]
        source: UiAssetDocumentError,
    },
    #[error("cached UI v2 asset document failed while {context}: {source}")]
    UiV2Document {
        context: &'static str,
        #[source]
        source: UiV2AssetDocumentError,
    },
    #[error("artifact cache serialization failed: {0}")]
    ArtifactCacheSerialize(#[source] bincode::Error),
    #[error("artifact cache deserialization failed: {0}")]
    ArtifactCacheDeserialize(#[source] bincode::Error),
    #[error("asset serialization failed: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
