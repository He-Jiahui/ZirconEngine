use thiserror::Error;

use super::WorldHandle;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum LevelManagerError {
    #[error("level runtime is no longer available")]
    RuntimeUnavailable,
    #[error("level handle space is exhausted")]
    HandleSpaceExhausted,
    #[error("default level creation failed: {reason}")]
    CreateFailed { reason: String },
    #[error("asset manager is unavailable: {reason}")]
    AssetManagerUnavailable { reason: String },
    #[error("asset manager has no active project generation")]
    ProjectUnavailable,
    #[error("active project root {active} does not match requested root {requested}")]
    ProjectRootMismatch { active: String, requested: String },
    #[error("invalid level resource locator {uri}: {reason}")]
    InvalidResourceLocator { uri: String, reason: String },
    #[error("failed to load level resource {uri}: {reason}")]
    LoadFailed { uri: String, reason: String },
    #[error("failed to save level {handle:?} to {uri}: {reason}")]
    SaveFailed {
        handle: WorldHandle,
        uri: String,
        reason: String,
    },
}
