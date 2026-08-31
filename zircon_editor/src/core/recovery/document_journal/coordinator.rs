use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::editing::engine::{
    DurableJournal, JournalCompactionReport, JournalDocumentKey, JournalReadReport, JournalWriter,
};
#[cfg(test)]
use crate::core::editing::engine::{
    EditorTransactionEngine, HistoryContextId, PreparedJournalRecord, TransactionId,
};
use crate::core::editor_message::DocumentId;

use super::{DocumentJournalAppend, DocumentJournalCoordinatorError};

/// Project-scoped bridge from session document handles to durable journal identities.
///
/// `DocumentId` is retained only in this process-local binding map. Disk identity always derives
/// from the validated project-relative source path stored in `JournalDocumentKey`.
pub struct DocumentJournalCoordinator {
    project_root: PathBuf,
    store: DurableJournal,
    bindings: Mutex<BTreeMap<DocumentId, Arc<BoundDocumentJournal>>>,
}

struct BoundDocumentJournal {
    key: JournalDocumentKey,
    // This is acquired before engine materialization so later committed callbacks cannot write
    // ahead of an earlier callback that is still preparing immutable durable bytes.
    append_gate: Mutex<()>,
    slot: Mutex<DocumentJournalSlot>,
}

#[derive(Default)]
struct DocumentJournalSlot {
    active: bool,
    writer: Option<JournalWriter>,
}

impl DocumentJournalCoordinator {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        Self {
            store: DurableJournal::new(project_root.clone()),
            project_root,
            bindings: Mutex::new(BTreeMap::new()),
        }
    }

    /// Returns the immutable project authority that owns all bound document journals.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Binds a scene source through this coordinator's project root.
    ///
    /// Durable identity derives from the physical project-relative source path, never from a
    /// session-local `DocumentId` or a logical `res://` URI.
    pub fn bind_project_document(
        &self,
        document: DocumentId,
        source_path: &Path,
    ) -> Result<(), DocumentJournalCoordinatorError> {
        let project_relative_source =
            source_path.strip_prefix(&self.project_root).map_err(|_| {
                DocumentJournalCoordinatorError::SourceOutsideProject {
                    project_root: self.project_root.clone(),
                    source_path: source_path.to_path_buf(),
                }
            })?;
        self.bind_document(document, project_relative_source)
    }

    pub fn bind_document(
        &self,
        document: DocumentId,
        project_relative_source: &Path,
    ) -> Result<(), DocumentJournalCoordinatorError> {
        let key = JournalDocumentKey::from_project_relative_path(project_relative_source)
            .map_err(DocumentJournalCoordinatorError::DocumentKey)?;
        let mut bindings = self.lock_bindings();
        if let Some(bound) = bindings.get(&document) {
            if bound.key == key {
                return Ok(());
            }
            return Err(DocumentJournalCoordinatorError::BindingConflict {
                document: document.value(),
                bound_source: bound.key.source_path().to_path_buf(),
                requested_source: key.source_path().to_path_buf(),
            });
        }
        bindings.insert(
            document,
            Arc::new(BoundDocumentJournal {
                key,
                append_gate: Mutex::new(()),
                slot: Mutex::new(DocumentJournalSlot {
                    active: true,
                    writer: None,
                }),
            }),
        );
        Ok(())
    }

    pub fn unbind_document(&self, document: DocumentId) -> bool {
        let mut bindings = self.lock_bindings();
        let Some(bound) = bindings.get(&document).cloned() else {
            return false;
        };
        let _append_guard = lock(&bound.append_gate);
        let mut slot = lock(&bound.slot);
        slot.active = false;
        slot.writer = None;
        bindings.remove(&document);
        true
    }

    pub fn journal_path(
        &self,
        document: DocumentId,
    ) -> Result<PathBuf, DocumentJournalCoordinatorError> {
        let bound = self.bound(document)?;
        Ok(self.store.path_for(&bound.key))
    }

    /// Exercises the coordinator's writer ownership in lib tests.
    ///
    /// Production durable publication is intentionally unavailable until the transaction engine
    /// owns immutable capture at its commit linearization point.
    #[cfg(test)]
    pub(crate) fn append_for_test(
        &self,
        engine: &EditorTransactionEngine,
        document: DocumentId,
        transaction: TransactionId,
    ) -> Result<DocumentJournalAppend, DocumentJournalCoordinatorError> {
        let bound = self.bound(document)?;
        let _append_guard = lock(&bound.append_gate);
        let transaction_journal = engine
            .journal_transaction(HistoryContextId::Document(document), transaction)
            .map_err(|source| {
                DocumentJournalCoordinatorError::transaction(document.value(), transaction, source)
            })?;
        let prepared = PreparedJournalRecord::prepare(&transaction_journal).map_err(|source| {
            DocumentJournalCoordinatorError::prepared(document.value(), transaction, source)
        })?;
        let mut slot = lock(&bound.slot);
        if !slot.active {
            return Err(DocumentJournalCoordinatorError::DocumentNotBound {
                document: document.value(),
            });
        }
        if slot.writer.is_none() {
            let writer = self.store.open(&bound.key).map_err(|source| {
                DocumentJournalCoordinatorError::durable(document.value(), source)
            })?;
            slot.writer = Some(writer);
        }
        let sequence = match slot.writer.as_mut() {
            Some(writer) => writer.append_prepared(prepared).map_err(|source| {
                DocumentJournalCoordinatorError::durable(document.value(), source)
            })?,
            None => {
                return Err(DocumentJournalCoordinatorError::WriterUnavailable {
                    document: document.value(),
                });
            }
        };
        Ok(DocumentJournalAppend::new(sequence))
    }

    /// Drops the active file handle before atomically replacing a snapshot-covered prefix.
    pub fn compact_covered_prefix(
        &self,
        document: DocumentId,
        covered_through: u64,
    ) -> Result<JournalCompactionReport, DocumentJournalCoordinatorError> {
        let bound = self.bound(document)?;
        let _append_guard = lock(&bound.append_gate);
        let mut slot = lock(&bound.slot);
        if !slot.active {
            return Err(DocumentJournalCoordinatorError::DocumentNotBound {
                document: document.value(),
            });
        }
        slot.writer = None;
        self.store
            .compact_covered_prefix(&bound.key, covered_through)
            .map_err(|source| DocumentJournalCoordinatorError::durable(document.value(), source))
    }

    pub fn read_document(
        &self,
        document: DocumentId,
    ) -> Result<JournalReadReport, DocumentJournalCoordinatorError> {
        let bound = self.bound(document)?;
        let _append_guard = lock(&bound.append_gate);
        let slot = lock(&bound.slot);
        if !slot.active {
            return Err(DocumentJournalCoordinatorError::DocumentNotBound {
                document: document.value(),
            });
        }
        self.store
            .read(&bound.key)
            .map_err(|source| DocumentJournalCoordinatorError::durable(document.value(), source))
    }

    fn bound(
        &self,
        document: DocumentId,
    ) -> Result<Arc<BoundDocumentJournal>, DocumentJournalCoordinatorError> {
        self.lock_bindings().get(&document).cloned().ok_or(
            DocumentJournalCoordinatorError::DocumentNotBound {
                document: document.value(),
            },
        )
    }

    fn lock_bindings(&self) -> MutexGuard<'_, BTreeMap<DocumentId, Arc<BoundDocumentJournal>>> {
        lock(&self.bindings)
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
