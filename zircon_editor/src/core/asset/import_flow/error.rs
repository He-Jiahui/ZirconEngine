use thiserror::Error;
use zircon_runtime::asset::{AssetUri, AssetUuid};

use crate::core::asset::EditorAssetIndexError;
use crate::core::jobs::{JobSubmitError, MutexGroupError};

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EditorAssetImportExecutionError {
    #[error("runtime asset import did not commit a status for {uri}")]
    RuntimeDidNotCommit { uri: AssetUri },
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum EditorAssetImportSubmitError {
    #[error("asset import URI is not present in the runtime registry projection: {uri}")]
    AssetNotIndexed { uri: AssetUri },
    #[error("asset import admission is pending for {uri}")]
    AdmissionPending { uri: AssetUri },
    #[error("asset import UUID lifecycle transition is pending for {uri}")]
    UuidLifecycleTransitionPending { uri: AssetUri },
    #[error("asset import UUID lifecycle state was inconsistent for {uri}")]
    UuidLifecycleStateInconsistent { uri: AssetUri },
    #[error("runtime asset registry generation changed repeatedly while submitting {uri}")]
    RegistryGenerationSuperseded { uri: AssetUri },
    #[error("asset import mutex-group identity space is exhausted")]
    MutexGroupIdentityExhausted,
    #[error("asset import flight identity space is exhausted")]
    FlightIdentityExhausted,
    #[error("asset import UUID lifecycle identity space is exhausted")]
    UuidLifecycleIdentityExhausted,
    #[error("asset import active-flight count is exhausted for UUID {uuid}")]
    UuidActiveFlightCountExhausted { uuid: AssetUuid },
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
