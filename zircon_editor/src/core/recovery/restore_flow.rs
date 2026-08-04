use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::SystemTime;

use thiserror::Error;

use super::{AutosaveDocumentId, SessionLockInspection, SessionLockRecord};

/// One document whose autosave may be recovered after an unclean editor exit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestoreCandidate {
    document: AutosaveDocumentId,
    source_path: PathBuf,
    autosave_path: PathBuf,
    source_modified_at: Option<SystemTime>,
    autosave_modified_at: SystemTime,
}

impl RestoreCandidate {
    pub fn new(
        document: AutosaveDocumentId,
        source_path: impl Into<PathBuf>,
        autosave_path: impl Into<PathBuf>,
        source_modified_at: Option<SystemTime>,
        autosave_modified_at: SystemTime,
    ) -> Self {
        Self {
            document,
            source_path: source_path.into(),
            autosave_path: autosave_path.into(),
            source_modified_at,
            autosave_modified_at,
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

    pub fn should_offer_recovery(&self) -> bool {
        self.source_modified_at
            .is_none_or(|source_modified_at| self.autosave_modified_at > source_modified_at)
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
        residual_lock: SessionLockRecord,
    },
    RecoveryRequired {
        residual_lock: SessionLockRecord,
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

    pub fn residual_lock(&self) -> Option<&SessionLockRecord> {
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
    /// Prompts only after a residual lock and an autosave newer than its source are both present.
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
}
