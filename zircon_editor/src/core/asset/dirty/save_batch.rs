use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use thiserror::Error;

use crate::core::editing::engine::{EditCommandError, EditorTransactionEngine, HistorySaveToken};
use crate::core::editor_message::DocumentId;
use crate::core::extension::{DocumentToolkitSnapshot, ToolkitInstanceId};

use super::{DirtyDocumentSnapshot, DirtyRegistry};

#[derive(Clone, Debug)]
pub struct SaveDirtyViewCandidate {
    snapshot: DirtyDocumentSnapshot,
    toolkit_instance: ToolkitInstanceId,
    save_token: HistorySaveToken,
    resource_key: String,
    estimated_bytes: u64,
    writable: bool,
    references_valid: bool,
}

impl SaveDirtyViewCandidate {
    pub fn new(
        snapshot: DirtyDocumentSnapshot,
        toolkit_instance: ToolkitInstanceId,
        save_token: HistorySaveToken,
        resource_key: impl Into<String>,
        estimated_bytes: u64,
    ) -> Self {
        Self {
            snapshot,
            toolkit_instance,
            save_token,
            resource_key: resource_key.into(),
            estimated_bytes,
            writable: true,
            references_valid: true,
        }
    }

    pub fn with_writable(mut self, writable: bool) -> Self {
        self.writable = writable;
        self
    }

    pub fn with_references_valid(mut self, references_valid: bool) -> Self {
        self.references_valid = references_valid;
        self
    }
}

#[derive(Clone, Debug)]
pub struct SaveDirtyViewIntent {
    snapshot: DirtyDocumentSnapshot,
    toolkit_instance: ToolkitInstanceId,
    save_token: HistorySaveToken,
    resource_key: String,
    estimated_bytes: u64,
}

impl SaveDirtyViewIntent {
    pub fn document_id(&self) -> DocumentId {
        self.snapshot.document()
    }

    pub fn dirty_generation(&self) -> u64 {
        self.snapshot.generation()
    }

    pub fn toolkit_instance(&self) -> &ToolkitInstanceId {
        &self.toolkit_instance
    }

    pub fn resource_key(&self) -> &str {
        &self.resource_key
    }

    pub fn estimated_bytes(&self) -> u64 {
        self.estimated_bytes
    }
}

#[derive(Clone, Debug)]
pub struct SaveDirtyViewsRequest {
    toolkit_generation: u64,
    total_estimated_bytes: u64,
    intents: Arc<[SaveDirtyViewIntent]>,
}

impl SaveDirtyViewsRequest {
    pub fn prepare(
        toolkits: &DocumentToolkitSnapshot,
        candidates: impl IntoIterator<Item = SaveDirtyViewCandidate>,
    ) -> Result<Self, SaveDirtyViewsPreflightReport> {
        let mut candidates = candidates.into_iter().collect::<Vec<_>>();
        candidates.sort_by_key(|candidate| candidate.snapshot.document());
        let mut seen = BTreeSet::new();
        let mut failures = Vec::new();
        let mut total_estimated_bytes = 0_u64;
        let toolkit_by_document = toolkits
            .descriptors()
            .iter()
            .map(|descriptor| (descriptor.document_id(), descriptor))
            .collect::<BTreeMap<_, _>>();

        for candidate in &candidates {
            let document = candidate.snapshot.document();
            if !seen.insert(document) {
                failures.push(SaveDirtyViewsPreflightError::new(
                    document,
                    SaveDirtyViewsPreflightErrorKind::DuplicateDocument,
                ));
            }
            if !candidate.snapshot.is_dirty() {
                failures.push(SaveDirtyViewsPreflightError::new(
                    document,
                    SaveDirtyViewsPreflightErrorKind::DocumentClean,
                ));
            }
            let toolkit = toolkit_by_document.get(&document).copied();
            match toolkit {
                None => failures.push(SaveDirtyViewsPreflightError::new(
                    document,
                    SaveDirtyViewsPreflightErrorKind::ToolkitMissing,
                )),
                Some(toolkit) if toolkit.instance_id() != &candidate.toolkit_instance => {
                    failures.push(SaveDirtyViewsPreflightError::new(
                        document,
                        SaveDirtyViewsPreflightErrorKind::ToolkitInstanceMismatch,
                    ));
                }
                Some(_) => {}
            }
            if candidate.resource_key.trim().is_empty()
                || candidate.resource_key.trim() != candidate.resource_key
            {
                failures.push(SaveDirtyViewsPreflightError::new(
                    document,
                    SaveDirtyViewsPreflightErrorKind::InvalidResourceKey,
                ));
            }
            if !candidate.writable {
                failures.push(SaveDirtyViewsPreflightError::new(
                    document,
                    SaveDirtyViewsPreflightErrorKind::ReadOnly,
                ));
            }
            if !candidate.references_valid {
                failures.push(SaveDirtyViewsPreflightError::new(
                    document,
                    SaveDirtyViewsPreflightErrorKind::ReferencePolicyRejected,
                ));
            }
            match total_estimated_bytes.checked_add(candidate.estimated_bytes) {
                Some(total) => total_estimated_bytes = total,
                None => failures.push(SaveDirtyViewsPreflightError::new(
                    document,
                    SaveDirtyViewsPreflightErrorKind::EstimatedBytesOverflow,
                )),
            }
        }

        if !failures.is_empty() {
            return Err(SaveDirtyViewsPreflightReport {
                failures: failures.into(),
            });
        }
        Ok(Self {
            toolkit_generation: toolkits.generation(),
            total_estimated_bytes,
            intents: candidates
                .into_iter()
                .map(|candidate| SaveDirtyViewIntent {
                    snapshot: candidate.snapshot,
                    toolkit_instance: candidate.toolkit_instance,
                    save_token: candidate.save_token,
                    resource_key: candidate.resource_key,
                    estimated_bytes: candidate.estimated_bytes,
                })
                .collect::<Vec<_>>()
                .into(),
        })
    }

    pub fn toolkit_generation(&self) -> u64 {
        self.toolkit_generation
    }

    pub fn total_estimated_bytes(&self) -> u64 {
        self.total_estimated_bytes
    }

    pub fn intents(&self) -> &[SaveDirtyViewIntent] {
        &self.intents
    }

    pub fn apply_completions(
        self,
        completions: impl IntoIterator<Item = (DocumentId, SaveDirtyViewCompletion)>,
        dirty: &DirtyRegistry,
        transactions: &EditorTransactionEngine,
    ) -> Result<SaveDirtyViewsResult, SaveDirtyViewsApplyError> {
        let expected = self
            .intents
            .iter()
            .map(SaveDirtyViewIntent::document_id)
            .collect::<BTreeSet<_>>();
        let mut completions_by_document = BTreeMap::new();
        for (document, completion) in completions {
            if !expected.contains(&document) {
                return Err(SaveDirtyViewsApplyError::UnknownCompletion { document });
            }
            if completions_by_document
                .insert(document, completion)
                .is_some()
            {
                return Err(SaveDirtyViewsApplyError::DuplicateCompletion { document });
            }
        }

        let mut outcomes = Vec::with_capacity(self.intents.len());
        for intent in self.intents.iter().cloned() {
            let document = intent.document_id();
            let completion = completions_by_document
                .remove(&document)
                .unwrap_or_else(|| {
                    SaveDirtyViewCompletion::Failed(SaveDirtyViewFailure::new(
                        SaveDirtyViewFailureKind::MissingCompletion,
                        "save adapter did not return a terminal completion",
                    ))
                });
            let status = apply_one_completion(intent, completion, dirty, transactions);
            outcomes.push(SaveDirtyViewOutcome {
                document,
                dirty_generation: status.0,
                status: status.1,
            });
        }
        Ok(SaveDirtyViewsResult {
            toolkit_generation: self.toolkit_generation,
            outcomes: outcomes.into(),
        })
    }
}

fn apply_one_completion(
    intent: SaveDirtyViewIntent,
    completion: SaveDirtyViewCompletion,
    dirty: &DirtyRegistry,
    transactions: &EditorTransactionEngine,
) -> (u64, SaveDirtyViewOutcomeStatus) {
    let document = intent.document_id();
    let generation = intent.dirty_generation();
    let SaveDirtyViewCompletion::Saved { written_bytes } = completion else {
        let status = match completion {
            SaveDirtyViewCompletion::Failed(failure) => SaveDirtyViewOutcomeStatus::Failed(failure),
            SaveDirtyViewCompletion::Cancelled => SaveDirtyViewOutcomeStatus::Cancelled,
            SaveDirtyViewCompletion::Saved { .. } => unreachable!(),
        };
        return (generation, status);
    };
    if let Err(error) = transactions.mark_saved_if_unchanged(
        crate::core::editing::engine::HistoryContextId::Document(document),
        intent.save_token,
    ) {
        if matches!(error, EditCommandError::HistoryChangedDuringSave { .. }) {
            return (generation, SaveDirtyViewOutcomeStatus::StaleGeneration);
        }
        return (
            generation,
            SaveDirtyViewOutcomeStatus::Failed(SaveDirtyViewFailure::new(
                SaveDirtyViewFailureKind::CompletionApply,
                error.to_string(),
            )),
        );
    }

    match dirty.clear_saved_external_effects(&intent.snapshot) {
        Ok(true) => {}
        Ok(false) => return (generation, SaveDirtyViewOutcomeStatus::StaleGeneration),
        Err(error) => return completion_apply_failure(generation, error),
    }
    (
        generation,
        SaveDirtyViewOutcomeStatus::Saved { written_bytes },
    )
}

fn completion_apply_failure(
    generation: u64,
    error: impl ToString,
) -> (u64, SaveDirtyViewOutcomeStatus) {
    (
        generation,
        SaveDirtyViewOutcomeStatus::Failed(SaveDirtyViewFailure::new(
            SaveDirtyViewFailureKind::CompletionApply,
            error.to_string(),
        )),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveDirtyViewCompletion {
    Saved { written_bytes: u64 },
    Failed(SaveDirtyViewFailure),
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveDirtyViewFailureKind {
    Admission,
    Serialize,
    Write,
    Import,
    Refresh,
    CompletionApply,
    MissingCompletion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveDirtyViewFailure {
    kind: SaveDirtyViewFailureKind,
    message: String,
}

impl SaveDirtyViewFailure {
    pub fn new(kind: SaveDirtyViewFailureKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> SaveDirtyViewFailureKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveDirtyViewOutcome {
    document: DocumentId,
    dirty_generation: u64,
    status: SaveDirtyViewOutcomeStatus,
}

impl SaveDirtyViewOutcome {
    pub fn document_id(&self) -> DocumentId {
        self.document
    }

    pub fn dirty_generation(&self) -> u64 {
        self.dirty_generation
    }

    pub fn status(&self) -> &SaveDirtyViewOutcomeStatus {
        &self.status
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveDirtyViewOutcomeStatus {
    Saved { written_bytes: u64 },
    Failed(SaveDirtyViewFailure),
    Cancelled,
    StaleGeneration,
}

#[derive(Clone, Debug)]
pub struct SaveDirtyViewsResult {
    toolkit_generation: u64,
    outcomes: Arc<[SaveDirtyViewOutcome]>,
}

impl SaveDirtyViewsResult {
    pub fn toolkit_generation(&self) -> u64 {
        self.toolkit_generation
    }

    pub fn outcomes(&self) -> &[SaveDirtyViewOutcome] {
        &self.outcomes
    }

    pub fn retry_documents(&self) -> impl Iterator<Item = DocumentId> + '_ {
        self.outcomes.iter().filter_map(|outcome| {
            (!matches!(outcome.status, SaveDirtyViewOutcomeStatus::Saved { .. }))
                .then_some(outcome.document)
        })
    }

    pub fn all_saved(&self) -> bool {
        self.outcomes
            .iter()
            .all(|outcome| matches!(outcome.status, SaveDirtyViewOutcomeStatus::Saved { .. }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveDirtyViewsPreflightErrorKind {
    DuplicateDocument,
    DocumentClean,
    ToolkitMissing,
    ToolkitInstanceMismatch,
    InvalidResourceKey,
    ReadOnly,
    ReferencePolicyRejected,
    EstimatedBytesOverflow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveDirtyViewsPreflightError {
    document: DocumentId,
    kind: SaveDirtyViewsPreflightErrorKind,
}

impl SaveDirtyViewsPreflightError {
    fn new(document: DocumentId, kind: SaveDirtyViewsPreflightErrorKind) -> Self {
        Self { document, kind }
    }

    pub fn document_id(&self) -> DocumentId {
        self.document
    }

    pub fn kind(&self) -> SaveDirtyViewsPreflightErrorKind {
        self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveDirtyViewsPreflightReport {
    failures: Arc<[SaveDirtyViewsPreflightError]>,
}

impl SaveDirtyViewsPreflightReport {
    pub fn failures(&self) -> &[SaveDirtyViewsPreflightError] {
        &self.failures
    }
}

#[derive(Debug, Error)]
pub enum SaveDirtyViewsApplyError {
    #[error("save completion references unknown document {document:?}")]
    UnknownCompletion { document: DocumentId },
    #[error("save completion contains duplicate document {document:?}")]
    DuplicateCompletion { document: DocumentId },
}

#[cfg(test)]
mod tests;
