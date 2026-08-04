use thiserror::Error;

use super::AssetImporterRegistryError;
use crate::asset::assets::ProjectDocumentError;
use crate::asset::assets::{
    FontAssetError, UiAssetDocumentError, UiIconAssetDocumentError, UiThemeAssetDocumentError,
    UiV2AssetDocumentError,
};
#[cfg(feature = "text")]
use crate::asset::assets::{FontMetadataParseError, FontSourceDecodeError};
use crate::asset::project::ProjectManifestError;
use crate::asset::registry::AssetRegistryError;
use crate::asset::ReferenceResolutionError;
use crate::core::framework::animation::AnimationAssetError;
use crate::core::resource::{ResourceLocator, ResourceLocatorError};

#[derive(Debug, Error)]
pub enum AssetImportError {
    #[error(transparent)]
    ProjectDocument(#[from] ProjectDocumentError),
    #[error(transparent)]
    ReferenceResolution(#[from] ReferenceResolutionError),
    #[error(transparent)]
    ProjectManifest(#[from] ProjectManifestError),
    #[error("asset I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("asset uri error: {0}")]
    Uri(#[from] ResourceLocatorError),
    #[error("asset source {path} is not valid UTF-8: {source}")]
    SourceTextDecode {
        path: std::path::PathBuf,
        #[source]
        source: std::string::FromUtf8Error,
    },
    #[error("asset parse failed: {0}")]
    Parse(String),
    #[error(
        "artifact raw payload requires {raw_bytes} bytes, exceeding the {limit_bytes}-byte read limit"
    )]
    ArtifactRawPayloadLimitExceeded { raw_bytes: u64, limit_bytes: u64 },
    #[error("authoring asset {path} requires an explicit project registry resolver")]
    ProjectContextRequired { path: std::path::PathBuf },
    #[error("project has no registered manifest asset roots")]
    MissingProjectAssetRoot,
    #[error("project asset root {root} escapes project root {project_root}")]
    ProjectAssetRootOutsideProject {
        project_root: std::path::PathBuf,
        root: std::path::PathBuf,
    },
    #[error("canonicalize project root {path} failed: {source}")]
    CanonicalProjectRoot {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("canonicalize project asset root {path} failed: {source}")]
    CanonicalProjectAssetRoot {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("canonical project asset root {asset_root} escapes project root {project_root}")]
    CanonicalProjectAssetRootEscape {
        project_root: std::path::PathBuf,
        asset_root: std::path::PathBuf,
    },
    #[error("project asset root {root} is registered more than once")]
    DuplicateProjectAssetRoot { root: std::path::PathBuf },
    #[error("project asset uri {uri} resolves from both {first} and {second}")]
    DuplicateProjectAssetUri {
        uri: ResourceLocator,
        first: std::path::PathBuf,
        second: std::path::PathBuf,
    },
    #[error("project asset uri {uri} does not exist in any registered manifest root")]
    MissingProjectAssetUri { uri: ResourceLocator },
    #[error("project asset uri {uri} exists in multiple registered manifest roots: {paths:?}")]
    AmbiguousProjectAssetUri {
        uri: ResourceLocator,
        paths: Vec<std::path::PathBuf>,
    },
    #[error("targeted import for {uri} requires a full generation scan: {reason}")]
    TargetedImportRequiresFullScan {
        uri: ResourceLocator,
        reason: String,
    },
    #[error("source path {path} is outside all registered manifest asset roots")]
    SourceOutsideProjectAssetRoots { path: std::path::PathBuf },
    #[error("source path {path} belongs to overlapping registered project roots: {roots:?}")]
    AmbiguousProjectSourcePath {
        path: std::path::PathBuf,
        roots: Vec<std::path::PathBuf>,
    },
    #[error("project asset scan rejected symbolic link or Windows reparse point at {path}")]
    UnsafeProjectAssetLink { path: std::path::PathBuf },
    #[error("font asset document failed: {0}")]
    FontDocument(#[source] FontAssetError),
    #[error("font source {path} could not be read: {source}")]
    FontSourceIo {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[cfg(feature = "text")]
    #[error("font source {path} could not be decoded: {source}")]
    FontSourceDecode {
        path: std::path::PathBuf,
        #[source]
        source: FontSourceDecodeError,
    },
    #[cfg(feature = "text")]
    #[error("font source {path} metadata is invalid: {source}")]
    FontMetadata {
        path: std::path::PathBuf,
        #[source]
        source: FontMetadataParseError,
    },
    #[error("font asset {manifest_path} has invalid source path: {reason}")]
    FontSourcePath {
        manifest_path: std::path::PathBuf,
        reason: &'static str,
    },
    #[error("unsupported asset format: {0}")]
    UnsupportedFormat(String),
    #[error("wgsl validation failed: {0}")]
    ShaderValidation(String),
    #[error("asset schema migration failed: {0}")]
    SchemaMigration(String),
    #[error("animation asset decode failed: {0}")]
    AnimationAsset(#[from] AnimationAssetError),
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
    #[error("asset registry index failed: {0}")]
    RegistryIndex(#[from] AssetRegistryError),
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
    #[error("asset JSON deserialization failed while {context}: {source}")]
    JsonDeserialize {
        context: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("cached TOML datetime `{value}` is invalid: {source}")]
    CachedTomlDatetime {
        value: String,
        #[source]
        source: toml::value::DatetimeParseError,
    },
    #[error("cached JSON number `{value}` is invalid because JSON numbers must be finite")]
    CachedJsonNonFiniteNumber { value: String },
    #[error("cached JSON number `{value}` is invalid: {source}")]
    CachedJsonNumberParse {
        value: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("ui asset document failed while {context}: {source}")]
    UiDocument {
        context: &'static str,
        #[source]
        source: UiAssetDocumentError,
    },
    #[error(".zui asset document failed while {context}: {source}")]
    UiV2Document {
        context: &'static str,
        #[source]
        source: UiV2AssetDocumentError,
    },
    #[error("ui theme asset document failed while {context}: {source}")]
    UiThemeDocument {
        context: &'static str,
        #[source]
        source: UiThemeAssetDocumentError,
    },
    #[error("ui icon asset document failed while {context}: {source}")]
    UiIconDocument {
        context: &'static str,
        #[source]
        source: UiIconAssetDocumentError,
    },
    #[error("artifact cache serialization failed: {0}")]
    ArtifactCacheSerialize(#[source] bincode::Error),
    #[error("artifact cache deserialization failed: {0}")]
    ArtifactCacheDeserialize(#[source] bincode::Error),
    #[error("asset serialization failed: {0}")]
    SerdeJson(#[from] serde_json::Error),
}
