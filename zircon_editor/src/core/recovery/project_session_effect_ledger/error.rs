use std::io;
use std::path::PathBuf;

use thiserror::Error;

use super::{
    ProjectSessionEffect, ProjectSessionEffectDisposition, ProjectSessionEffectLedgerPhase,
};

#[derive(Debug, Error)]
pub(crate) enum ProjectSessionEffectLedgerError {
    #[error("session effect `{effect:?}` is not valid during `{phase:?}`")]
    EffectNotAllowed {
        phase: ProjectSessionEffectLedgerPhase,
        effect: ProjectSessionEffect,
    },
    #[error("session effect `{effect:?}` cannot transition from `{current:?}` to `{requested:?}`")]
    InvalidEffectTransition {
        effect: ProjectSessionEffect,
        current: Option<ProjectSessionEffectDisposition>,
        requested: ProjectSessionEffectDisposition,
    },
    #[error("session effect ledger cannot transition from `{current:?}` to `{requested:?}`")]
    InvalidPhaseTransition {
        current: ProjectSessionEffectLedgerPhase,
        requested: ProjectSessionEffectLedgerPhase,
    },
    #[error("session effect ledger phase `{phase:?}` is missing committed effects {effects:?}")]
    MissingCommittedEffects {
        phase: ProjectSessionEffectLedgerPhase,
        effects: Vec<ProjectSessionEffect>,
    },
    #[error("session effect ledger phase `{phase:?}` still owns unsettled effects {effects:?}")]
    UnsettledEffects {
        phase: ProjectSessionEffectLedgerPhase,
        effects: Vec<ProjectSessionEffect>,
    },
    #[error("session effect ledger at `{path}` uses unsupported schema version {actual}")]
    UnsupportedSchemaVersion { path: PathBuf, actual: u32 },
    #[error("session effect ledger at `{path}` belongs to a different operation")]
    OperationMismatch { path: PathBuf },
    #[error(
        "session effect ledger at `{path}` is {actual_bytes} bytes, exceeding the {max_bytes}-byte limit"
    )]
    RecordTooLarge {
        path: PathBuf,
        actual_bytes: usize,
        max_bytes: usize,
    },
    #[error("session effect ledger at `{path}` is invalid: {message}")]
    InvalidRecord { path: PathBuf, message: String },
    #[error("failed to encode session effect ledger at `{path}`: {source}")]
    Encode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to {operation} session effect ledger at `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
