use std::{
    collections::BTreeMap,
    fmt,
    ops::Range,
    sync::{Arc, Mutex},
};

use zircon_runtime_interface::ui::text::{UiTextDocumentId, UiTextDocumentRevision};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    text::{UiTextByteSelection, UiTextEditKind, UiTextEditReceipt, UiTextEditSource},
};

use super::{
    PreparedTextDocumentChange, PreparedTextDocumentReplace, TextDocument, TextDocumentEditError,
    TextDocumentEditOutcome, TextDocumentEditReceipt, TextDocumentReceiptProjectionError,
    TextDocumentSnapshotLease,
};

/// Explicit admission policy for one surface/session document store.
///
/// There is intentionally no `Default`: product owners must select and qualify every limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextDocumentStoreLimits {
    pub(crate) max_documents: usize,
    pub(crate) max_document_bytes: usize,
    pub(crate) max_total_document_bytes: usize,
    pub(crate) max_replacement_bytes: usize,
    pub(crate) max_retained_source_bytes_per_document: usize,
    pub(crate) max_total_retained_source_bytes: usize,
    pub(crate) max_addition_sources_per_document: usize,
    pub(crate) max_pieces_per_document: usize,
    pub(crate) max_current_snapshot_bytes: usize,
    pub(crate) max_active_snapshot_leases: usize,
    pub(crate) max_active_snapshot_lease_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OpenedTextDocument {
    pub(crate) document_id: UiTextDocumentId,
    pub(crate) revision: UiTextDocumentRevision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextDocumentAdmissionFailure {
    DocumentCount,
    DocumentBytes,
    TotalDocumentBytes,
    ReplacementBytes,
    DocumentRetainedSourceBytes,
    TotalRetainedSourceBytes,
    AdditionSources,
    Pieces,
    CurrentSnapshotBytes,
    ActiveSnapshotLeaseCount,
    ActiveSnapshotLeaseBytes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextDocumentStoreError {
    UnknownDocument,
    StaleRevision {
        expected: UiTextDocumentRevision,
        actual: UiTextDocumentRevision,
    },
    DocumentOwnerExhausted,
    SnapshotLeaseBudgetUnavailable,
    AdmissionDenied(TextDocumentAdmissionFailure),
    ReceiptProjection(TextDocumentReceiptProjectionError),
    Edit(TextDocumentEditError),
}

impl From<TextDocumentEditError> for TextDocumentStoreError {
    fn from(error: TextDocumentEditError) -> Self {
        Self::Edit(error)
    }
}

impl From<TextDocumentReceiptProjectionError> for TextDocumentStoreError {
    fn from(error: TextDocumentReceiptProjectionError) -> Self {
        Self::ReceiptProjection(error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TextDocumentStoreEditCommit {
    Unchanged {
        document_id: UiTextDocumentId,
        revision: UiTextDocumentRevision,
    },
    Changed {
        internal_receipt: TextDocumentEditReceipt,
        public_receipt: UiTextEditReceipt,
    },
}

enum PreparedTextDocumentStoreEditKind {
    Unchanged {
        document_id: UiTextDocumentId,
        revision: UiTextDocumentRevision,
    },
    Changed {
        prepared: PreparedTextDocumentChange,
        public_receipt: UiTextEditReceipt,
        next_residency: TextDocumentStoreResidency,
    },
}

#[must_use = "prepared document edits must be committed or explicitly discarded"]
pub(crate) struct PreparedTextDocumentStoreEdit<'store> {
    document: &'store mut TextDocument,
    residency: &'store mut TextDocumentStoreResidency,
    kind: PreparedTextDocumentStoreEditKind,
}

impl PreparedTextDocumentStoreEdit<'_> {
    pub(crate) const fn public_receipt(&self) -> Option<&UiTextEditReceipt> {
        match &self.kind {
            PreparedTextDocumentStoreEditKind::Unchanged { .. } => None,
            PreparedTextDocumentStoreEditKind::Changed { public_receipt, .. } => {
                Some(public_receipt)
            }
        }
    }

    pub(crate) fn commit(self) -> TextDocumentStoreEditCommit {
        match self.kind {
            PreparedTextDocumentStoreEditKind::Unchanged {
                document_id,
                revision,
            } => TextDocumentStoreEditCommit::Unchanged {
                document_id,
                revision,
            },
            PreparedTextDocumentStoreEditKind::Changed {
                prepared,
                public_receipt,
                next_residency,
            } => {
                let internal_receipt = self.document.commit_prepared_change(prepared);
                *self.residency = next_residency;
                TextDocumentStoreEditCommit::Changed {
                    internal_receipt,
                    public_receipt,
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextDocumentStoreReport {
    pub(crate) document_count: usize,
    pub(crate) current_document_bytes: usize,
    pub(crate) retained_source_bytes: usize,
    pub(crate) current_snapshot_bytes: usize,
    pub(crate) active_snapshot_lease_count: usize,
    pub(crate) active_snapshot_lease_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TextDocumentStoreResidency {
    current_document_bytes: usize,
    retained_source_bytes: usize,
    current_snapshot_bytes: usize,
}

impl TextDocumentStoreResidency {
    fn after_open(self, source_bytes: usize) -> Option<Self> {
        Some(Self {
            current_document_bytes: self.current_document_bytes.checked_add(source_bytes)?,
            retained_source_bytes: self.retained_source_bytes.checked_add(source_bytes)?,
            current_snapshot_bytes: self.current_snapshot_bytes,
        })
    }

    fn after_close(self, document: &super::TextDocumentStorageReport) -> Option<Self> {
        Some(Self {
            current_document_bytes: self.current_document_bytes.checked_sub(document.byte_len)?,
            retained_source_bytes: self.retained_source_bytes.checked_sub(
                document
                    .original_bytes
                    .checked_add(document.addition_bytes)?,
            )?,
            current_snapshot_bytes: self
                .current_snapshot_bytes
                .checked_sub(document.flattened_snapshot_bytes)?,
        })
    }

    fn after_change(
        self,
        current: &super::TextDocumentStorageReport,
        prepared: &PreparedTextDocumentChange,
    ) -> Option<Self> {
        Some(Self {
            current_document_bytes: self
                .current_document_bytes
                .checked_sub(current.byte_len)?
                .checked_add(prepared.byte_len())?,
            retained_source_bytes: self
                .retained_source_bytes
                .checked_add(prepared.added_source_bytes())?,
            current_snapshot_bytes: self
                .current_snapshot_bytes
                .checked_sub(current.flattened_snapshot_bytes)?,
        })
    }

    fn after_snapshot(self, materialized_bytes: usize) -> Option<Self> {
        Some(Self {
            current_snapshot_bytes: self
                .current_snapshot_bytes
                .checked_add(materialized_bytes)?,
            ..self
        })
    }
}

struct PreparedTextDocumentStoreReplace {
    prepared: PreparedTextDocumentReplace,
    next_residency: TextDocumentStoreResidency,
}

#[derive(Default)]
struct SnapshotLeaseUsage {
    count: usize,
    bytes: usize,
}

pub(crate) struct ManagedTextDocumentSnapshotLease {
    inner: TextDocumentSnapshotLease,
    usage: Arc<Mutex<SnapshotLeaseUsage>>,
}

impl ManagedTextDocumentSnapshotLease {
    pub(crate) const fn document_id(&self) -> UiTextDocumentId {
        self.inner.document_id()
    }

    pub(crate) const fn revision(&self) -> UiTextDocumentRevision {
        self.inner.revision()
    }

    pub(crate) fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

impl fmt::Debug for ManagedTextDocumentSnapshotLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ManagedTextDocumentSnapshotLease")
            .field("document_id", &self.document_id())
            .field("revision", &self.revision())
            .field("byte_len", &self.as_str().len())
            .finish()
    }
}

impl Drop for ManagedTextDocumentSnapshotLease {
    fn drop(&mut self) {
        let byte_len = self.as_str().len();
        let mut usage = self
            .usage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        usage.count = usage.count.saturating_sub(1);
        usage.bytes = usage.bytes.saturating_sub(byte_len);
    }
}

/// Mutable document authority scoped to one product surface or editing session.
pub(crate) struct TextDocumentStore {
    limits: TextDocumentStoreLimits,
    next_owner: Option<u64>,
    documents: BTreeMap<UiTextDocumentId, TextDocument>,
    residency: TextDocumentStoreResidency,
    snapshot_usage: Arc<Mutex<SnapshotLeaseUsage>>,
}

impl fmt::Debug for TextDocumentStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TextDocumentStore")
            .field("limits", &self.limits)
            .field("report", &self.report())
            .finish()
    }
}

impl TextDocumentStore {
    pub(crate) fn with_limits(limits: TextDocumentStoreLimits) -> Self {
        Self {
            limits,
            next_owner: Some(1),
            documents: BTreeMap::new(),
            residency: TextDocumentStoreResidency::default(),
            snapshot_usage: Arc::new(Mutex::new(SnapshotLeaseUsage::default())),
        }
    }

    pub(crate) const fn limits(&self) -> TextDocumentStoreLimits {
        self.limits
    }

    pub(crate) fn open(
        &mut self,
        source: impl Into<Arc<str>>,
    ) -> Result<OpenedTextDocument, TextDocumentStoreError> {
        let source = source.into();
        self.admit_open(source.len())?;
        let next_residency =
            self.residency
                .after_open(source.len())
                .ok_or(TextDocumentStoreError::Edit(
                    TextDocumentEditError::StorageInvariant,
                ))?;
        let owner = self
            .next_owner
            .ok_or(TextDocumentStoreError::DocumentOwnerExhausted)?;
        self.next_owner = owner.checked_add(1);
        let document = TextDocument::new(owner, source);
        let opened = OpenedTextDocument {
            document_id: document.document_id(),
            revision: UiTextDocumentRevision::new(0),
        };
        self.documents.insert(opened.document_id, document);
        self.residency = next_residency;
        Ok(opened)
    }

    pub(crate) fn close(&mut self, document_id: UiTextDocumentId) -> bool {
        let Some(document) = self.documents.get(&document_id) else {
            return false;
        };
        let Some(next_residency) = self.residency.after_close(&document.storage_report()) else {
            return false;
        };
        self.documents.remove(&document_id);
        self.residency = next_residency;
        true
    }

    pub(crate) fn replace(
        &mut self,
        document_id: UiTextDocumentId,
        expected_revision: UiTextDocumentRevision,
        range: Range<usize>,
        replacement: &str,
    ) -> Result<TextDocumentEditOutcome, TextDocumentStoreError> {
        let prepared =
            self.prepare_admitted_replace(document_id, expected_revision, range, replacement)?;
        let next_residency = prepared.next_residency;
        let document = self
            .documents
            .get_mut(&document_id)
            .ok_or(TextDocumentStoreError::UnknownDocument)?;
        let outcome = document.commit_replace(prepared.prepared)?;
        if matches!(outcome, TextDocumentEditOutcome::Changed(_)) {
            self.residency = next_residency;
        }
        Ok(outcome)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn replace_with_receipt(
        &mut self,
        document_id: UiTextDocumentId,
        expected_revision: UiTextDocumentRevision,
        range: Range<usize>,
        replacement: &str,
        node_id: UiNodeId,
        source: UiTextEditSource,
        kind: UiTextEditKind,
        selection: UiTextByteSelection,
    ) -> Result<TextDocumentStoreEditCommit, TextDocumentStoreError> {
        Ok(self
            .prepare_replace_with_receipt(
                document_id,
                expected_revision,
                range,
                replacement,
                node_id,
                source,
                kind,
                selection,
            )?
            .commit())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare_replace_with_receipt(
        &mut self,
        document_id: UiTextDocumentId,
        expected_revision: UiTextDocumentRevision,
        range: Range<usize>,
        replacement: &str,
        node_id: UiNodeId,
        source: UiTextEditSource,
        kind: UiTextEditKind,
        selection: UiTextByteSelection,
    ) -> Result<PreparedTextDocumentStoreEdit<'_>, TextDocumentStoreError> {
        let prepared =
            self.prepare_admitted_replace(document_id, expected_revision, range, replacement)?;
        let kind = match prepared.prepared {
            PreparedTextDocumentReplace::Unchanged { document_id, key } => {
                PreparedTextDocumentStoreEditKind::Unchanged {
                    document_id,
                    revision: UiTextDocumentRevision::new(key.revision()),
                }
            }
            PreparedTextDocumentReplace::Changed(change) => {
                let public_receipt = change.project_public(node_id, source, kind, selection)?;
                PreparedTextDocumentStoreEditKind::Changed {
                    prepared: change,
                    public_receipt,
                    next_residency: prepared.next_residency,
                }
            }
        };
        let (documents, residency) = (&mut self.documents, &mut self.residency);
        let document = documents
            .get_mut(&document_id)
            .ok_or(TextDocumentStoreError::UnknownDocument)?;
        Ok(PreparedTextDocumentStoreEdit {
            document,
            residency,
            kind,
        })
    }

    fn prepare_admitted_replace(
        &self,
        document_id: UiTextDocumentId,
        expected_revision: UiTextDocumentRevision,
        range: Range<usize>,
        replacement: &str,
    ) -> Result<PreparedTextDocumentStoreReplace, TextDocumentStoreError> {
        if replacement.len() > self.limits.max_replacement_bytes {
            return Err(TextDocumentStoreError::AdmissionDenied(
                TextDocumentAdmissionFailure::ReplacementBytes,
            ));
        }
        let document = self
            .documents
            .get(&document_id)
            .ok_or(TextDocumentStoreError::UnknownDocument)?;
        let actual_revision = UiTextDocumentRevision::new(document.key().revision());
        if expected_revision != actual_revision {
            return Err(TextDocumentStoreError::StaleRevision {
                expected: expected_revision,
                actual: actual_revision,
            });
        }
        let document_report = document.storage_report();
        let prepared = document.prepare_replace(document.key(), range, replacement)?;
        let next_residency = if let PreparedTextDocumentReplace::Changed(change) = &prepared {
            self.admit_replace(&self.residency, &document_report, change)?;
            self.residency
                .after_change(&document_report, change)
                .ok_or(TextDocumentStoreError::Edit(
                    TextDocumentEditError::StorageInvariant,
                ))?
        } else {
            self.residency
        };
        Ok(PreparedTextDocumentStoreReplace {
            prepared,
            next_residency,
        })
    }

    pub(crate) fn snapshot(
        &mut self,
        document_id: UiTextDocumentId,
        expected_revision: UiTextDocumentRevision,
    ) -> Result<ManagedTextDocumentSnapshotLease, TextDocumentStoreError> {
        let document = self
            .documents
            .get(&document_id)
            .ok_or(TextDocumentStoreError::UnknownDocument)?;
        let actual_revision = UiTextDocumentRevision::new(document.key().revision());
        if expected_revision != actual_revision {
            return Err(TextDocumentStoreError::StaleRevision {
                expected: expected_revision,
                actual: actual_revision,
            });
        }
        let document_report = document.storage_report();
        let new_snapshot_bytes = (!document_report.has_flattened_snapshot)
            .then_some(document_report.byte_len)
            .unwrap_or(0);
        admit_total(
            self.residency.current_snapshot_bytes,
            new_snapshot_bytes,
            self.limits.max_current_snapshot_bytes,
            TextDocumentAdmissionFailure::CurrentSnapshotBytes,
        )?;
        let next_residency = self.residency.after_snapshot(new_snapshot_bytes).ok_or(
            TextDocumentStoreError::Edit(TextDocumentEditError::StorageInvariant),
        )?;

        {
            let usage = self
                .snapshot_usage
                .lock()
                .map_err(|_| TextDocumentStoreError::SnapshotLeaseBudgetUnavailable)?;
            admit_total(
                usage.count,
                1,
                self.limits.max_active_snapshot_leases,
                TextDocumentAdmissionFailure::ActiveSnapshotLeaseCount,
            )?;
            admit_total(
                usage.bytes,
                document.len(),
                self.limits.max_active_snapshot_lease_bytes,
                TextDocumentAdmissionFailure::ActiveSnapshotLeaseBytes,
            )?;
        }
        let lease = document.snapshot_lease();
        {
            let mut usage = self
                .snapshot_usage
                .lock()
                .map_err(|_| TextDocumentStoreError::SnapshotLeaseBudgetUnavailable)?;
            usage.count += 1;
            usage.bytes += lease.as_str().len();
        }
        self.residency = next_residency;

        Ok(ManagedTextDocumentSnapshotLease {
            inner: lease,
            usage: Arc::clone(&self.snapshot_usage),
        })
    }

    pub(crate) fn source_range(
        &self,
        document_id: UiTextDocumentId,
        expected_revision: UiTextDocumentRevision,
        range: Range<usize>,
    ) -> Result<String, TextDocumentStoreError> {
        let document = self
            .documents
            .get(&document_id)
            .ok_or(TextDocumentStoreError::UnknownDocument)?;
        let actual_revision = UiTextDocumentRevision::new(document.key().revision());
        if expected_revision != actual_revision {
            return Err(TextDocumentStoreError::StaleRevision {
                expected: expected_revision,
                actual: actual_revision,
            });
        }
        document.snapshot_range(range).map_err(Into::into)
    }

    pub(crate) fn retained_grapheme_count(
        &mut self,
        document_id: UiTextDocumentId,
        expected_revision: UiTextDocumentRevision,
        range: Range<usize>,
    ) -> Result<usize, TextDocumentStoreError> {
        let document = self
            .documents
            .get(&document_id)
            .ok_or(TextDocumentStoreError::UnknownDocument)?;
        let actual_revision = UiTextDocumentRevision::new(document.key().revision());
        if expected_revision != actual_revision {
            return Err(TextDocumentStoreError::StaleRevision {
                expected: expected_revision,
                actual: actual_revision,
            });
        }
        document.validate_range(&range)?;
        let document_report = document.storage_report();
        let new_snapshot_bytes = (!document_report.has_flattened_snapshot)
            .then_some(document_report.byte_len)
            .unwrap_or(0);
        admit_total(
            self.residency.current_snapshot_bytes,
            new_snapshot_bytes,
            self.limits.max_current_snapshot_bytes,
            TextDocumentAdmissionFailure::CurrentSnapshotBytes,
        )?;
        let next_residency = self.residency.after_snapshot(new_snapshot_bytes).ok_or(
            TextDocumentStoreError::Edit(TextDocumentEditError::StorageInvariant),
        )?;

        let result = self
            .documents
            .get_mut(&document_id)
            .ok_or(TextDocumentStoreError::UnknownDocument)?
            .retained_grapheme_count(range);
        self.residency = next_residency;
        result.map_err(Into::into)
    }

    pub(crate) fn report(&self) -> TextDocumentStoreReport {
        let mut report = TextDocumentStoreReport {
            document_count: self.documents.len(),
            current_document_bytes: self.residency.current_document_bytes,
            retained_source_bytes: self.residency.retained_source_bytes,
            current_snapshot_bytes: self.residency.current_snapshot_bytes,
            ..TextDocumentStoreReport::default()
        };
        let usage = self
            .snapshot_usage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        report.active_snapshot_lease_count = usage.count;
        report.active_snapshot_lease_bytes = usage.bytes;
        report
    }

    fn admit_open(&self, source_bytes: usize) -> Result<(), TextDocumentStoreError> {
        if self.documents.len() >= self.limits.max_documents {
            return Err(TextDocumentStoreError::AdmissionDenied(
                TextDocumentAdmissionFailure::DocumentCount,
            ));
        }
        admit_value(
            source_bytes,
            self.limits.max_document_bytes,
            TextDocumentAdmissionFailure::DocumentBytes,
        )?;
        admit_value(
            source_bytes,
            self.limits.max_retained_source_bytes_per_document,
            TextDocumentAdmissionFailure::DocumentRetainedSourceBytes,
        )?;
        admit_total(
            self.residency.current_document_bytes,
            source_bytes,
            self.limits.max_total_document_bytes,
            TextDocumentAdmissionFailure::TotalDocumentBytes,
        )?;
        admit_total(
            self.residency.retained_source_bytes,
            source_bytes,
            self.limits.max_total_retained_source_bytes,
            TextDocumentAdmissionFailure::TotalRetainedSourceBytes,
        )
    }

    fn admit_replace(
        &self,
        totals: &TextDocumentStoreResidency,
        current: &super::TextDocumentStorageReport,
        prepared: &PreparedTextDocumentChange,
    ) -> Result<(), TextDocumentStoreError> {
        let byte_len = prepared.byte_len();
        admit_value(
            byte_len,
            self.limits.max_document_bytes,
            TextDocumentAdmissionFailure::DocumentBytes,
        )?;
        admit_replacement_total(
            totals.current_document_bytes,
            current.byte_len,
            byte_len,
            self.limits.max_total_document_bytes,
            TextDocumentAdmissionFailure::TotalDocumentBytes,
        )?;

        let retained_source_bytes = current
            .original_bytes
            .saturating_add(current.addition_bytes)
            .saturating_add(prepared.added_source_bytes());
        admit_value(
            retained_source_bytes,
            self.limits.max_retained_source_bytes_per_document,
            TextDocumentAdmissionFailure::DocumentRetainedSourceBytes,
        )?;
        admit_total(
            totals.retained_source_bytes,
            prepared.added_source_bytes(),
            self.limits.max_total_retained_source_bytes,
            TextDocumentAdmissionFailure::TotalRetainedSourceBytes,
        )?;
        admit_value(
            prepared.addition_source_count(current.addition_source_count),
            self.limits.max_addition_sources_per_document,
            TextDocumentAdmissionFailure::AdditionSources,
        )?;
        admit_value(
            prepared.piece_count(),
            self.limits.max_pieces_per_document,
            TextDocumentAdmissionFailure::Pieces,
        )
    }
}

fn admit_value(
    value: usize,
    limit: usize,
    failure: TextDocumentAdmissionFailure,
) -> Result<(), TextDocumentStoreError> {
    if value > limit {
        return Err(TextDocumentStoreError::AdmissionDenied(failure));
    }
    Ok(())
}

fn admit_total(
    current: usize,
    added: usize,
    limit: usize,
    failure: TextDocumentAdmissionFailure,
) -> Result<(), TextDocumentStoreError> {
    let Some(total) = current.checked_add(added) else {
        return Err(TextDocumentStoreError::AdmissionDenied(failure));
    };
    admit_value(total, limit, failure)
}

fn admit_replacement_total(
    current: usize,
    removed: usize,
    added: usize,
    limit: usize,
    failure: TextDocumentAdmissionFailure,
) -> Result<(), TextDocumentStoreError> {
    let Some(total) = current
        .checked_sub(removed)
        .and_then(|retained| retained.checked_add(added))
    else {
        return Err(TextDocumentStoreError::AdmissionDenied(failure));
    };
    admit_value(total, limit, failure)
}
