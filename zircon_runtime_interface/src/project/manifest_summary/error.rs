use thiserror::Error;

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
    #[error("project manifest has an invalid value: {message}")]
    InvalidValue { message: String },
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
