use thiserror::Error;
use zircon_runtime::asset::AssetUri;

use crate::core::asset::EditorAssetIndexError;
use crate::core::jobs::{JobSubmitError, MutexGroupError};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EditorAssetImportSubmitError {
    #[error("asset import URI is not present in the runtime registry projection: {uri}")]
    AssetNotIndexed { uri: AssetUri },
    #[error("asset import admission reached its retained flight limit of {limit}")]
    FlightLimitReached { limit: usize },
    #[error(
        "asset import admission byte budget exceeded: limit={limit}, current={current}, requested={requested}"
    )]
    ByteLimitExceeded {
        limit: usize,
        current: usize,
        requested: usize,
    },
    #[error(
        "asset import admission is stalled beyond its oldest-flight age budget of {max_age_ms} ms"
    )]
    OldestFlightAgeExceeded { max_age_ms: u64 },
    #[error(transparent)]
    MutexGroup(#[from] MutexGroupError),
    #[error(transparent)]
    Index(#[from] EditorAssetIndexError),
    #[error(transparent)]
    Job(#[from] JobSubmitError),
}
