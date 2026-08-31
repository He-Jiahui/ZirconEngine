//! Immutable transaction intent and append-only state transitions.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub(super) const JOURNAL_VERSION: u32 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionFault {
    None,
    #[cfg(any(test, feature = "test-support"))]
    BeforeCommit(usize),
    #[cfg(any(test, feature = "test-support"))]
    RestoreFailure {
        commit_index: usize,
        restore_index: usize,
    },
    #[cfg(any(test, feature = "test-support"))]
    FailRollbackTransition {
        commit_index: usize,
        restore_index: usize,
    },
    #[cfg(any(test, feature = "test-support"))]
    CrashAfterCommit(usize),
    #[cfg(any(test, feature = "test-support"))]
    CrashAfterAllCommitted,
    #[cfg(any(test, feature = "test-support"))]
    FailCommitPointWrite,
    #[cfg(any(test, feature = "test-support"))]
    FailCommitPointSync,
    #[cfg(any(test, feature = "test-support"))]
    CrashAfterCleanup,
    #[cfg(any(test, feature = "test-support"))]
    FailCleanupTransition,
    #[cfg(any(test, feature = "test-support"))]
    FailCommittedCleanup,
    #[cfg(any(test, feature = "test-support"))]
    CrashAfterRollbackCompleted {
        commit_index: usize,
    },
    #[cfg(any(test, feature = "test-support"))]
    FailRollbackJournalDelete {
        commit_index: usize,
    },
    #[cfg(any(test, feature = "test-support"))]
    FailStageWrite(usize),
    #[cfg(any(test, feature = "test-support"))]
    FailStagingDirectorySync(usize),
    #[cfg(any(test, feature = "test-support"))]
    FailBackupCopy(usize),
    #[cfg(any(test, feature = "test-support"))]
    FailRetiredBackupSync(usize),
    #[cfg(any(test, feature = "test-support"))]
    CrashAfterStaging(usize),
    #[cfg(any(test, feature = "test-support"))]
    CrashAfterTargetReplace(usize),
    #[cfg(any(test, feature = "test-support"))]
    FailAfterTargetReplace(usize),
    #[cfg(any(test, feature = "test-support"))]
    CrashAfterRetiredDelete(usize),
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum JournalState {
    Intent,
    Prepared,
    Committing,
    Committed,
    RollingBack,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum JournalPhase {
    Intent,
    CleanupIntent,
    Active,
    RollbackCompleted,
    CleanupRollback,
    AllCommitted,
    Cleanup,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TransactionJournal {
    pub(super) version: u32,
    pub(super) tag: String,
    pub(super) transaction_id: String,
    pub(super) documents: Vec<JournalIntent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) transitions: Vec<JournalTransition>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JournalIntent {
    pub(super) target: PathBuf,
    pub(super) staging: PathBuf,
    pub(super) backup: PathBuf,
    pub(super) rollback_staging: PathBuf,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) retirements: Vec<JournalRetirement>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JournalRetirement {
    pub(super) path: PathBuf,
    pub(super) backup: PathBuf,
    pub(super) rollback_staging: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JournalTransition {
    pub(super) phase: JournalPhase,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) document_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) state: Option<JournalState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) target_existed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) original_digest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) new_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) retired_digests: Vec<String>,
}

#[derive(Debug)]
pub(super) struct FoldedTransactionJournal {
    pub(super) tag: String,
    pub(super) transaction_id: String,
    pub(super) phase: JournalPhase,
    pub(super) documents: Vec<JournalDocument>,
}

#[derive(Debug)]
pub struct JournalDocument {
    pub(super) state: JournalState,
    pub(super) target_existed: Option<bool>,
    pub(super) original_digest: Option<String>,
    pub(super) new_digest: Option<String>,
    pub(super) retired_digests: Vec<String>,
    pub(super) target: PathBuf,
    pub(super) staging: PathBuf,
    pub(super) backup: PathBuf,
    pub(super) rollback_staging: PathBuf,
    pub(super) retirements: Vec<JournalRetirement>,
}

impl JournalDocument {
    pub fn target(&self) -> &std::path::Path {
        &self.target
    }

    pub fn retired_paths(&self) -> impl Iterator<Item = &std::path::Path> {
        self.retirements
            .iter()
            .map(|retirement| retirement.path.as_path())
    }
}

impl TransactionJournal {
    pub(super) fn fold(&self) -> Result<FoldedTransactionJournal, String> {
        let mut phase = JournalPhase::Intent;
        let mut documents = self
            .documents
            .iter()
            .map(|intent| JournalDocument {
                state: JournalState::Intent,
                target_existed: None,
                original_digest: None,
                new_digest: None,
                retired_digests: Vec::new(),
                target: intent.target.clone(),
                staging: intent.staging.clone(),
                backup: intent.backup.clone(),
                rollback_staging: intent.rollback_staging.clone(),
                retirements: intent.retirements.clone(),
            })
            .collect::<Vec<_>>();

        for (offset, transition) in self.transitions.iter().enumerate() {
            let label = format!("journal transition {offset}");
            if let Some(index) = transition.document_index {
                let document = documents.get_mut(index).ok_or_else(|| {
                    format!("{label} references a document outside the immutable intent")
                })?;
                fold_document_transition(&mut phase, document, transition, &label)?;
            } else {
                fold_phase_transition(&mut phase, &mut documents, transition, &label)?;
            }
        }

        Ok(FoldedTransactionJournal {
            tag: self.tag.clone(),
            transaction_id: self.transaction_id.clone(),
            phase,
            documents,
        })
    }
}

fn fold_document_transition(
    phase: &mut JournalPhase,
    document: &mut JournalDocument,
    transition: &JournalTransition,
    label: &str,
) -> Result<(), String> {
    match transition.state {
        Some(JournalState::Prepared) => {
            if *phase != JournalPhase::Intent || transition.phase != JournalPhase::Intent {
                return Err(format!("{label} prepares a document after activation"));
            }
            if document.state != JournalState::Intent {
                return Err(format!("{label} prepares a document more than once"));
            }
            let target_existed = transition
                .target_existed
                .ok_or_else(|| format!("{label} omits target origin evidence"))?;
            let new_digest = transition
                .new_digest
                .clone()
                .ok_or_else(|| format!("{label} omits staged digest evidence"))?;
            if target_existed != transition.original_digest.is_some() {
                return Err(format!("{label} has inconsistent target origin evidence"));
            }
            if document.retirements.len() != transition.retired_digests.len() {
                return Err(format!("{label} has incomplete retired-file evidence"));
            }
            document.state = JournalState::Prepared;
            document.target_existed = Some(target_existed);
            document.original_digest = transition.original_digest.clone();
            document.new_digest = Some(new_digest);
            document.retired_digests = transition.retired_digests.clone();
        }
        Some(JournalState::Committing) => {
            reject_transition_evidence(transition, label)?;
            if *phase != JournalPhase::Active
                || transition.phase != JournalPhase::Active
                || document.state != JournalState::Prepared
            {
                return Err(format!("{label} begins an invalid live write"));
            }
            document.state = JournalState::Committing;
        }
        Some(JournalState::Committed) => {
            reject_transition_evidence(transition, label)?;
            if *phase != JournalPhase::Active
                || transition.phase != JournalPhase::Active
                || document.state != JournalState::Committing
            {
                return Err(format!("{label} completes an invalid live write"));
            }
            document.state = JournalState::Committed;
        }
        Some(JournalState::RollingBack) => {
            reject_transition_evidence(transition, label)?;
            if *phase != JournalPhase::Active
                || transition.phase != JournalPhase::Active
                || !matches!(
                    document.state,
                    JournalState::Committing | JournalState::Committed | JournalState::RollingBack
                )
            {
                return Err(format!("{label} begins invalid rollback"));
            }
            document.state = JournalState::RollingBack;
        }
        Some(JournalState::Intent) | None => {
            return Err(format!("{label} has no valid document transition"));
        }
    }
    Ok(())
}

fn fold_phase_transition(
    phase: &mut JournalPhase,
    documents: &mut [JournalDocument],
    transition: &JournalTransition,
    label: &str,
) -> Result<(), String> {
    reject_transition_evidence(transition, label)?;
    if transition.state.is_some() {
        return Err(format!("{label} assigns state without a document index"));
    }
    match transition.phase {
        JournalPhase::CleanupIntent => {
            if *phase != JournalPhase::Intent
                || documents.iter().any(|document| {
                    !matches!(
                        document.state,
                        JournalState::Intent | JournalState::Prepared
                    )
                })
            {
                return Err(format!("{label} cleans an activated transaction as intent"));
            }
        }
        JournalPhase::Active => {
            if *phase != JournalPhase::Intent
                || documents
                    .iter()
                    .any(|document| document.state != JournalState::Prepared)
            {
                return Err(format!("{label} activates an incomplete transaction"));
            }
        }
        JournalPhase::RollbackCompleted => {
            if *phase != JournalPhase::Active
                || documents
                    .iter()
                    .any(|document| document.state == JournalState::Intent)
            {
                return Err(format!("{label} records rollback before prepared evidence"));
            }
            for document in documents {
                document.state = JournalState::Prepared;
            }
        }
        JournalPhase::CleanupRollback => {
            if *phase != JournalPhase::RollbackCompleted {
                return Err(format!("{label} cleans rollback before completion"));
            }
        }
        JournalPhase::AllCommitted => {
            if *phase != JournalPhase::Active
                || documents
                    .iter()
                    .any(|document| document.state != JournalState::Committed)
            {
                return Err(format!("{label} completes before every document"));
            }
        }
        JournalPhase::Cleanup => {
            if *phase != JournalPhase::AllCommitted {
                return Err(format!("{label} cleans committed artifacts too early"));
            }
        }
        JournalPhase::Intent => {
            return Err(format!("{label} repeats immutable intent"));
        }
    }
    *phase = transition.phase;
    Ok(())
}

fn reject_transition_evidence(transition: &JournalTransition, label: &str) -> Result<(), String> {
    if transition.target_existed.is_some()
        || transition.original_digest.is_some()
        || transition.new_digest.is_some()
        || !transition.retired_digests.is_empty()
    {
        return Err(format!("{label} unexpectedly repeats prepared evidence"));
    }
    Ok(())
}
