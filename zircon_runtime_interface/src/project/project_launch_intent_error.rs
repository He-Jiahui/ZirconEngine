use std::path::PathBuf;

use thiserror::Error;

use super::ProjectNameError;

/// Rejects a launch request that cannot safely enter project preflight.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ProjectLaunchIntentError {
    #[error("project launch intent schema version {actual} is unsupported; expected {expected}")]
    UnsupportedSchemaVersion { expected: u32, actual: u32 },
    #[error("project launch intent open path must not be empty")]
    EmptyOpenPath,
    #[error("project launch intent create location must not be empty")]
    EmptyCreateLocation,
    #[error("project launch intent create name is invalid: {source}")]
    ProjectName {
        #[from]
        #[source]
        source: ProjectNameError,
    },
    #[error("project launch intent path {path:?} contains no textual path input")]
    NonTextPath { path: PathBuf },
}
