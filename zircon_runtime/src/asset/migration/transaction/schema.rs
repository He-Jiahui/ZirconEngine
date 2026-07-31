//! Durable transaction journal schema. Intent is immutable; transitions are appended and folded.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub(super) const JOURNAL_VERSION: u32 = 3;
pub(in crate::asset::migration) const JOURNAL_DIRECTORY: &str = "asset-migration";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::asset::migration) enum CommitFault {
    Never,
    #[cfg(test)]
    At(usize),
    #[cfg(test)]
    AtWithRestoreFailure {
        commit_index: usize,
        restore_index: usize,
    },
    #[cfg(test)]
    CrashAfter(usize),
    #[cfg(test)]
    CrashAfterAllCommitted,
    #[cfg(test)]
    CrashAfterCleanup,
    #[cfg(test)]
    CrashAfterRollbackCompleted {
        commit_index: usize,
    },
    #[cfg(test)]
    FailRollbackJournalDelete {
        commit_index: usize,
    },
    #[cfg(test)]
    FailStageWrite(usize),
    #[cfg(test)]
    FailBackupCopy(usize),
    #[cfg(test)]
    FailRetiredBackupSync(usize),
    #[cfg(test)]
    CrashAfterStaging(usize),
    #[cfg(test)]
    CrashAfterTargetReplace(usize),
    #[cfg(test)]
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
    pub(super) transaction_id: String,
    pub(super) documents: Vec<JournalIntent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) transitions: Vec<JournalTransition>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JournalIntent {
    pub(super) target: PathBuf,
    pub(super) staging: PathBuf,
    pub(super) backup: Option<PathBuf>,
    pub(super) retired_path: Option<PathBuf>,
    pub(super) retired_backup: Option<PathBuf>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) retired_digest: Option<String>,
}

#[derive(Debug)]
pub(super) struct FoldedTransactionJournal {
    pub(super) transaction_id: String,
    pub(super) phase: JournalPhase,
    pub(super) documents: Vec<JournalDocument>,
}

#[derive(Debug)]
pub(super) struct JournalDocument {
    pub(super) state: JournalState,
    pub(super) target_existed: Option<bool>,
    pub(super) original_digest: Option<String>,
    pub(super) new_digest: Option<String>,
    pub(super) retired_digest: Option<String>,
    pub(super) target: PathBuf,
    pub(super) staging: PathBuf,
    pub(super) backup: Option<PathBuf>,
    pub(super) retired_path: Option<PathBuf>,
    pub(super) retired_backup: Option<PathBuf>,
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
                retired_digest: None,
                target: intent.target.clone(),
                staging: intent.staging.clone(),
                backup: intent.backup.clone(),
                retired_path: intent.retired_path.clone(),
                retired_backup: intent.retired_backup.clone(),
            })
            .collect::<Vec<_>>();

        for (transition_offset, transition) in self.transitions.iter().enumerate() {
            let label = format!("journal transition {transition_offset}");
            if let Some(document_index) = transition.document_index {
                let document = documents.get_mut(document_index).ok_or_else(|| {
                    format!("{label} references a document outside the immutable intent")
                })?;
                match transition.state {
                    Some(JournalState::Prepared) => {
                        if phase != JournalPhase::Intent || transition.phase != JournalPhase::Intent
                        {
                            return Err(format!("{label} prepares a document after activation"));
                        }
                        if document.state != JournalState::Intent {
                            return Err(format!("{label} prepares a document more than once"));
                        }
                        let target_existed = transition
                            .target_existed
                            .ok_or_else(|| format!("{label} omits the target origin evidence"))?;
                        let new_digest = transition
                            .new_digest
                            .clone()
                            .ok_or_else(|| format!("{label} omits the staged artifact digest"))?;
                        if target_existed != transition.original_digest.is_some() {
                            return Err(format!("{label} has inconsistent target origin evidence"));
                        }
                        if document.retired_path.is_some() != transition.retired_digest.is_some() {
                            return Err(format!("{label} has incomplete retired-sidecar evidence"));
                        }
                        document.state = JournalState::Prepared;
                        document.target_existed = Some(target_existed);
                        document.original_digest = transition.original_digest.clone();
                        document.new_digest = Some(new_digest);
                        document.retired_digest = transition.retired_digest.clone();
                    }
                    Some(JournalState::Committing) => {
                        reject_transition_evidence(transition, &label)?;
                        if phase != JournalPhase::Active || transition.phase != JournalPhase::Active
                        {
                            return Err(format!(
                                "{label} begins a live write outside the active phase"
                            ));
                        }
                        if document.state != JournalState::Prepared {
                            return Err(format!(
                                "{label} begins a live write without prepared evidence"
                            ));
                        }
                        document.state = JournalState::Committing;
                    }
                    Some(JournalState::Committed) => {
                        reject_transition_evidence(transition, &label)?;
                        if phase != JournalPhase::Active || transition.phase != JournalPhase::Active
                        {
                            return Err(format!(
                                "{label} completes a live write outside the active phase"
                            ));
                        }
                        if document.state != JournalState::Committing {
                            return Err(format!(
                                "{label} completes a live write without a commit barrier"
                            ));
                        }
                        document.state = JournalState::Committed;
                    }
                    Some(JournalState::RollingBack) => {
                        reject_transition_evidence(transition, &label)?;
                        if phase != JournalPhase::Active || transition.phase != JournalPhase::Active
                        {
                            return Err(format!(
                                "{label} begins rollback outside the active phase"
                            ));
                        }
                        if !matches!(
                            document.state,
                            JournalState::Committing | JournalState::Committed
                        ) {
                            return Err(format!(
                                "{label} begins rollback without committed target evidence"
                            ));
                        }
                        document.state = JournalState::RollingBack;
                    }
                    Some(JournalState::Intent) | None => {
                        return Err(format!("{label} has no valid document state transition"));
                    }
                }
            } else {
                reject_transition_evidence(transition, &label)?;
                if transition.state.is_some() {
                    return Err(format!("{label} assigns a state without a document index"));
                }
                match transition.phase {
                    JournalPhase::Active => {
                        if phase != JournalPhase::Intent
                            || documents
                                .iter()
                                .any(|document| document.state != JournalState::Prepared)
                        {
                            return Err(format!("{label} activates an incomplete transaction"));
                        }
                    }
                    JournalPhase::RollbackCompleted => {
                        if phase != JournalPhase::Active
                            || documents
                                .iter()
                                .any(|document| document.state == JournalState::Intent)
                        {
                            return Err(format!("{label} records rollback before staged evidence"));
                        }
                        for document in &mut documents {
                            document.state = JournalState::Prepared;
                        }
                    }
                    JournalPhase::CleanupRollback => {
                        if phase != JournalPhase::RollbackCompleted {
                            return Err(format!(
                                "{label} cleans rollback before rollback completion"
                            ));
                        }
                    }
                    JournalPhase::AllCommitted => {
                        if phase != JournalPhase::Active
                            || documents
                                .iter()
                                .any(|document| document.state != JournalState::Committed)
                        {
                            return Err(format!(
                                "{label} completes before every document is committed"
                            ));
                        }
                    }
                    JournalPhase::Cleanup => {
                        if phase != JournalPhase::AllCommitted {
                            return Err(format!("{label} cleans committed artifacts too early"));
                        }
                    }
                    JournalPhase::Intent => {
                        return Err(format!("{label} redundantly records the immutable intent"));
                    }
                }
                phase = transition.phase;
            }
        }

        Ok(FoldedTransactionJournal {
            transaction_id: self.transaction_id.clone(),
            phase,
            documents,
        })
    }
}

fn reject_transition_evidence(transition: &JournalTransition, label: &str) -> Result<(), String> {
    if transition.target_existed.is_some()
        || transition.original_digest.is_some()
        || transition.new_digest.is_some()
        || transition.retired_digest.is_some()
    {
        return Err(format!("{label} unexpectedly repeats immutable evidence"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        JOURNAL_VERSION, JournalIntent, JournalPhase, JournalState, JournalTransition,
        TransactionJournal,
    };

    #[test]
    fn rolling_back_after_live_replace_folds_from_committing_state() {
        let journal = TransactionJournal {
            version: JOURNAL_VERSION,
            transaction_id: "1-1".to_owned(),
            documents: vec![JournalIntent {
                target: PathBuf::from("target.zmaterial"),
                staging: PathBuf::from(".target.zmaterial.zr-migrate-stage-1-1"),
                backup: Some(PathBuf::from(".target.zmaterial.zr-migrate-backup-1-1")),
                retired_path: None,
                retired_backup: None,
            }],
            transitions: vec![
                JournalTransition {
                    phase: JournalPhase::Intent,
                    document_index: Some(0),
                    state: Some(JournalState::Prepared),
                    target_existed: Some(true),
                    original_digest: Some("original".to_owned()),
                    new_digest: Some("new".to_owned()),
                    retired_digest: None,
                },
                JournalTransition {
                    phase: JournalPhase::Active,
                    document_index: None,
                    state: None,
                    target_existed: None,
                    original_digest: None,
                    new_digest: None,
                    retired_digest: None,
                },
                JournalTransition {
                    phase: JournalPhase::Active,
                    document_index: Some(0),
                    state: Some(JournalState::Committing),
                    target_existed: None,
                    original_digest: None,
                    new_digest: None,
                    retired_digest: None,
                },
                JournalTransition {
                    phase: JournalPhase::Active,
                    document_index: Some(0),
                    state: Some(JournalState::RollingBack),
                    target_existed: None,
                    original_digest: None,
                    new_digest: None,
                    retired_digest: None,
                },
            ],
        };

        let folded = journal
            .fold()
            .expect("rollback after a replaced target must remain recoverable");
        assert_eq!(folded.phase, JournalPhase::Active);
        assert_eq!(folded.documents[0].state, JournalState::RollingBack);
    }
}
