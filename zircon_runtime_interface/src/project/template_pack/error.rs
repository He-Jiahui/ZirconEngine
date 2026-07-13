use thiserror::Error;

use crate::project::{ProjectManifestSummaryError, RelPathError};

/// Typed failure while validating and rendering an embedded project template pack.
#[derive(Debug, Error)]
pub enum ProjectTemplatePackError {
    #[error("project name cannot be empty")]
    EmptyProjectName,
    #[error("template entry path is invalid: {source}")]
    InvalidEntryPath {
        #[from]
        #[source]
        source: RelPathError,
    },
    #[error("project template does not contain zircon-project.toml")]
    MissingManifest,
    #[error("project template manifest is not UTF-8: {source}")]
    ManifestUtf8 {
        #[source]
        source: std::str::Utf8Error,
    },
    #[error("project template manifest TOML is invalid: {source}")]
    ManifestToml {
        #[source]
        source: toml::de::Error,
    },
    #[error("project template manifest could not be encoded: {source}")]
    ManifestEncode {
        #[source]
        source: toml::ser::Error,
    },
    #[error("rendered project template manifest is invalid: {source}")]
    ManifestSummary {
        #[from]
        #[source]
        source: ProjectManifestSummaryError,
    },
}
