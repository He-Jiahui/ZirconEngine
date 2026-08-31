use thiserror::Error;

use crate::project::ProjectNameError;
use crate::serialization::MigrateError;

/// Typed failure produced by the lightweight project-manifest reader.
#[derive(Debug, Error)]
pub enum ProjectManifestSummaryError {
    #[error("project manifest is not UTF-8: {source}")]
    InvalidUtf8 {
        #[source]
        source: std::str::Utf8Error,
    },
    #[error("project manifest TOML is invalid: {source}")]
    InvalidToml {
        #[source]
        source: toml::de::Error,
    },
    #[error("project manifest has an invalid field shape: {source}")]
    InvalidShape {
        #[source]
        source: serde_json::Error,
    },
    #[error("project manifest exceeds {max} bytes (found {found})")]
    DocumentTooLarge { max: usize, found: usize },
    #[error("project manifest TOML nesting exceeds depth {max} (found {found})")]
    TomlNestingTooDeep { max: usize, found: usize },
    #[error("project manifest TOML tables exceed {max} cumulative entries (found {found})")]
    TooManyTomlTableEntries { max: usize, found: usize },
    #[error("project manifest TOML arrays exceed {max} cumulative items (found {found})")]
    TooManyTomlArrayItems { max: usize, found: usize },
    #[error("project manifest has an invalid value: {message}")]
    InvalidValue { message: String },
    #[error("project manifest project name is invalid: {source}")]
    InvalidProjectName {
        #[source]
        source: ProjectNameError,
    },
    #[error("project manifest declares {found} asset roots; maximum is {max}")]
    TooManyAssetRoots { max: usize, found: usize },
    #[error("project manifest declares duplicate normalized asset root {root}")]
    DuplicateAssetRoot { root: String },
    #[error("project manifest asset root {ancestor} contains nested root {descendant}")]
    OverlappingAssetRoots {
        ancestor: String,
        descendant: String,
    },
    #[error("project manifest engine_version_req {value:?} is invalid: {source}")]
    InvalidEngineVersionReq {
        value: String,
        #[source]
        source: semver::Error,
    },
    #[error("project manifest format_version must be an unsigned 32-bit integer")]
    InvalidFormatVersion,
    #[error("project manifest version {found} is newer than supported version {supported}")]
    FutureVersion { found: u32, supported: u32 },
    #[error(transparent)]
    Migration(#[from] MigrateError),
}
