use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use thiserror::Error;

use super::{
    AutosaveContentDigest, AutosaveDocumentId, AutosaveSourceDigest,
    ProjectSessionAdmissionRecordV1, SessionLockInspection,
};

/// The content relationship between an autosave commit and its authoritative source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreFreshness {
    SourceMissing,
    SnapshotAheadOfSource,
    SourceDiverged,
    SnapshotAlreadyCommitted,
}

impl RestoreFreshness {
    pub(crate) fn from_snapshot(
        captured_source: &AutosaveSourceDigest,
        committed_snapshot: &AutosaveContentDigest,
        current_source: &AutosaveSourceDigest,
    ) -> Self {
        match current_source {
            AutosaveSourceDigest::Missing => Self::SourceMissing,
            AutosaveSourceDigest::Present(current) if current == committed_snapshot => {
                Self::SnapshotAlreadyCommitted
            }
            AutosaveSourceDigest::Present(_) if current_source == captured_source => {
                Self::SnapshotAheadOfSource
            }
            AutosaveSourceDigest::Present(_) => Self::SourceDiverged,
        }
    }
}

/// One document whose autosave may be recovered after an unclean editor exit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestoreCandidate {
    document: AutosaveDocumentId,
    source_path: PathBuf,
    autosave_path: PathBuf,
    freshness: RestoreFreshness,
}

impl RestoreCandidate {
    pub fn new(
        document: AutosaveDocumentId,
        source_path: impl Into<PathBuf>,
        autosave_path: impl Into<PathBuf>,
        freshness: RestoreFreshness,
    ) -> Self {
        Self {
            document,
            source_path: source_path.into(),
            autosave_path: autosave_path.into(),
            freshness,
        }
    }

    pub fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub fn source_path(&self) -> &std::path::Path {
        &self.source_path
    }

    pub fn autosave_path(&self) -> &std::path::Path {
        &self.autosave_path
    }

    pub const fn freshness(&self) -> RestoreFreshness {
        self.freshness
    }

    pub fn should_offer_recovery(&self) -> bool {
        self.freshness != RestoreFreshness::SnapshotAlreadyCommitted
    }
}

/// The recovery UI must make one explicit choice for each candidate document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreAction {
    RestoreAutosave,
    DiscardAutosave,
    OpenComparison,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestoreResolution {
    document: AutosaveDocumentId,
    action: RestoreAction,
}

impl RestoreResolution {
    pub fn new(document: AutosaveDocumentId, action: RestoreAction) -> Self {
        Self { document, action }
    }

    pub fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub const fn action(&self) -> RestoreAction {
        self.action
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoreStartup {
    NoRecoveryNeeded,
    ResidualTakeoverRequired {
        residual_lock: ProjectSessionAdmissionRecordV1,
    },
    RecoveryRequired {
        residual_lock: ProjectSessionAdmissionRecordV1,
        candidates: Vec<RestoreCandidate>,
    },
}

impl RestoreStartup {
    pub fn candidates(&self) -> &[RestoreCandidate] {
        match self {
            Self::NoRecoveryNeeded => &[],
            Self::ResidualTakeoverRequired { .. } => &[],
            Self::RecoveryRequired { candidates, .. } => candidates,
        }
    }

    pub fn residual_lock(&self) -> Option<&ProjectSessionAdmissionRecordV1> {
        match self {
            Self::NoRecoveryNeeded => None,
            Self::ResidualTakeoverRequired { residual_lock }
            | Self::RecoveryRequired { residual_lock, .. } => Some(residual_lock),
        }
    }
}

/// A fully specified recovery decision for document lifecycle owners to execute.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestorePlan {
    resolutions: Vec<RestoreResolution>,
}

impl RestorePlan {
    pub fn resolutions(&self) -> &[RestoreResolution] {
        &self.resolutions
    }
}

pub struct RestoreFlow;

impl RestoreFlow {
    /// Prompts only after a residual lock and a committed snapshot remains recoverable.
    pub fn detect(
        lock: SessionLockInspection,
        candidates: impl IntoIterator<Item = RestoreCandidate>,
    ) -> Result<RestoreStartup, RestoreFlowError> {
        let SessionLockInspection::Residual(residual_lock) = lock else {
            return Ok(RestoreStartup::NoRecoveryNeeded);
        };
        let mut candidate_by_document = BTreeMap::new();
        for candidate in candidates {
            let document = candidate.document().clone();
            if candidate_by_document
                .insert(document.clone(), candidate)
                .is_some()
            {
                return Err(RestoreFlowError::DuplicateCandidate {
                    document: document.as_str().to_string(),
                });
            }
        }
        let candidates = candidate_by_document
            .into_values()
            .filter(RestoreCandidate::should_offer_recovery)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            Ok(RestoreStartup::ResidualTakeoverRequired { residual_lock })
        } else {
            Ok(RestoreStartup::RecoveryRequired {
                residual_lock,
                candidates,
            })
        }
    }

    /// Requires one and only one decision for every candidate before any source owner acts.
    pub fn plan(
        startup: &RestoreStartup,
        resolutions: impl IntoIterator<Item = RestoreResolution>,
    ) -> Result<RestorePlan, RestoreFlowError> {
        let required = startup
            .candidates()
            .iter()
            .map(|candidate| candidate.document().clone())
            .collect::<BTreeSet<_>>();
        let mut by_document = BTreeMap::new();
        for resolution in resolutions {
            let document = resolution.document().clone();
            if by_document.insert(document.clone(), resolution).is_some() {
                return Err(RestoreFlowError::DuplicateResolution {
                    document: document.as_str().to_string(),
                });
            }
        }
        let received = by_document.keys().cloned().collect::<BTreeSet<_>>();
        if received != required {
            if let Some(document) = required.difference(&received).next() {
                return Err(RestoreFlowError::MissingResolution {
                    document: document.as_str().to_string(),
                });
            }
            let document = received
                .difference(&required)
                .next()
                .expect("non-equal sets have one differing document");
            return Err(RestoreFlowError::UnexpectedResolution {
                document: document.as_str().to_string(),
            });
        }
        Ok(RestorePlan {
            resolutions: by_document.into_values().collect(),
        })
    }

    /// Builds a subset plan for retrying failed document executions.
    pub fn retry_plan(
        original: &RestorePlan,
        resolutions: impl IntoIterator<Item = RestoreResolution>,
    ) -> Result<Option<RestorePlan>, RestoreFlowError> {
        let original_by_document = original
            .resolutions()
            .iter()
            .map(|resolution| (resolution.document().clone(), resolution.action()))
            .collect::<BTreeMap<_, _>>();
        let mut retry_by_document = BTreeMap::new();
        for resolution in resolutions {
            let document = resolution.document().clone();
            let Some(original_action) = original_by_document.get(&document) else {
                return Err(RestoreFlowError::UnexpectedResolution {
                    document: document.as_str().to_string(),
                });
            };
            if *original_action != resolution.action() {
                return Err(RestoreFlowError::ChangedRetryAction {
                    document: document.as_str().to_string(),
                    original: *original_action,
                    retry: resolution.action(),
                });
            }
            if retry_by_document
                .insert(document.clone(), resolution)
                .is_some()
            {
                return Err(RestoreFlowError::DuplicateResolution {
                    document: document.as_str().to_string(),
                });
            }
        }
        Ok((!retry_by_document.is_empty()).then(|| RestorePlan {
            resolutions: retry_by_document.into_values().collect(),
        }))
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RestoreFlowError {
    #[error("recovery candidate `{document}` appears more than once")]
    DuplicateCandidate { document: String },
    #[error("recovery resolution `{document}` appears more than once")]
    DuplicateResolution { document: String },
    #[error("recovery decision is missing for `{document}`")]
    MissingResolution { document: String },
    #[error("recovery decision references unexpected document `{document}`")]
    UnexpectedResolution { document: String },
    #[error(
        "recovery retry for `{document}` changed the original action from {original:?} to {retry:?}"
    )]
    ChangedRetryAction {
        document: String,
        original: RestoreAction,
        retry: RestoreAction,
    },
}
