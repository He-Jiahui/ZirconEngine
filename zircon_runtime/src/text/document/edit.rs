use std::ops::Range;

use super::TextDocumentHardLineSpan;
use super::hard_line_model::PreparedHardLineEdit;
use super::index::PreparedTextDocumentSourceIndexEdit;
use super::storage::{TextDocument, TextDocumentPiece, TextDocumentPieceSource};
use crate::text::TextDocumentKey;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::UiTextByteRange,
    text::{
        UiTextByteSelection, UiTextChangedRanges, UiTextDocumentId, UiTextDocumentRevision,
        UiTextEditKind, UiTextEditReceipt, UiTextEditReceiptError, UiTextEditSource,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextDocumentDirtySpan {
    /// Source range in the revision before the edit.
    pub(crate) old: Range<usize>,
    /// Replacement range in the revision after the edit.
    pub(crate) new: Range<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextDocumentLengthDelta {
    Unchanged,
    Increased(usize),
    Decreased(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextDocumentEditReceipt {
    pub(crate) document_id: UiTextDocumentId,
    pub(crate) previous_key: TextDocumentKey,
    pub(crate) key: TextDocumentKey,
    pub(crate) dirty: TextDocumentDirtySpan,
    pub(crate) reanalyzed_hard_lines: TextDocumentHardLineSpan,
    pub(crate) length_delta: TextDocumentLengthDelta,
    pub(crate) previous_byte_len: usize,
    pub(crate) byte_len: usize,
}

impl TextDocumentEditReceipt {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn project_public(
        &self,
        node_id: UiNodeId,
        source: UiTextEditSource,
        kind: UiTextEditKind,
        selection: UiTextByteSelection,
    ) -> Result<UiTextEditReceipt, TextDocumentReceiptProjectionError> {
        if self.previous_key.owner() != self.key.owner() {
            return Err(TextDocumentReceiptProjectionError::DocumentOwnerChanged);
        }
        let previous_revision = self.previous_key.revision();
        if previous_revision.checked_add(1) != Some(self.key.revision()) {
            return Err(TextDocumentReceiptProjectionError::NonConsecutiveRevision);
        }

        let old = project_byte_range(&self.dirty.old)?;
        let new = project_byte_range(&self.dirty.new)?;
        if length_delta(self.dirty.old.len(), self.dirty.new.len()) != self.length_delta {
            return Err(TextDocumentReceiptProjectionError::InconsistentLengthDelta);
        }
        if length_delta(self.previous_byte_len, self.byte_len) != self.length_delta {
            return Err(TextDocumentReceiptProjectionError::InconsistentDocumentLength);
        }
        let public_new_document_len = u32::try_from(self.byte_len)
            .map_err(|_| TextDocumentReceiptProjectionError::DocumentLengthOverflow)?;
        let public_previous_document_len = u32::try_from(self.previous_byte_len)
            .map_err(|_| TextDocumentReceiptProjectionError::DocumentLengthOverflow)?;
        if old.end_byte > public_previous_document_len || new.end_byte > public_new_document_len {
            return Err(TextDocumentReceiptProjectionError::ChangedRangeOutOfBounds);
        }
        if selection.anchor_byte > public_new_document_len
            || selection.focus_byte > public_new_document_len
        {
            return Err(TextDocumentReceiptProjectionError::SelectionOutOfBounds);
        }

        UiTextEditReceipt::new(
            node_id,
            self.document_id,
            UiTextDocumentRevision::new(previous_revision),
            source,
            kind,
            UiTextChangedRanges { old, new },
            selection,
        )
        .map_err(TextDocumentReceiptProjectionError::InvalidPublicReceipt)
    }
}

pub(crate) enum PreparedTextDocumentReplace {
    Unchanged {
        document_id: UiTextDocumentId,
        key: TextDocumentKey,
    },
    Changed(PreparedTextDocumentChange),
}

pub(crate) struct PreparedTextDocumentChange {
    document_id: UiTextDocumentId,
    previous_key: TextDocumentKey,
    next_revision: u64,
    dirty: TextDocumentDirtySpan,
    reanalyzed_hard_lines: TextDocumentHardLineSpan,
    length_delta: TextDocumentLengthDelta,
    previous_byte_len: usize,
    byte_len: usize,
    replacement: Option<String>,
    pieces: Vec<TextDocumentPiece>,
    hard_line_edit: PreparedHardLineEdit,
    source_index_edit: Option<PreparedTextDocumentSourceIndexEdit>,
}

impl PreparedTextDocumentReplace {
    pub(crate) const fn document_id(&self) -> UiTextDocumentId {
        match self {
            Self::Unchanged { document_id, .. } => *document_id,
            Self::Changed(change) => change.document_id,
        }
    }

    pub(crate) const fn expected_key(&self) -> TextDocumentKey {
        match self {
            Self::Unchanged { key, .. } => *key,
            Self::Changed(change) => change.previous_key,
        }
    }
}

impl PreparedTextDocumentChange {
    pub(crate) const fn byte_len(&self) -> usize {
        self.byte_len
    }

    pub(crate) fn added_source_bytes(&self) -> usize {
        match &self.replacement {
            Some(replacement) => replacement.len(),
            None => 0,
        }
    }

    pub(crate) fn addition_source_count(&self, current_count: usize) -> usize {
        current_count.max(self.replacement.is_some() as usize)
    }

    pub(crate) fn piece_count(&self) -> usize {
        self.pieces.len()
    }

    pub(crate) fn project_public(
        &self,
        node_id: UiNodeId,
        source: UiTextEditSource,
        kind: UiTextEditKind,
        selection: UiTextByteSelection,
    ) -> Result<UiTextEditReceipt, TextDocumentReceiptProjectionError> {
        self.prospective_receipt()
            .project_public(node_id, source, kind, selection)
    }

    fn prospective_receipt(&self) -> TextDocumentEditReceipt {
        TextDocumentEditReceipt {
            document_id: self.document_id,
            previous_key: self.previous_key,
            key: TextDocumentKey::new(self.previous_key.owner(), self.next_revision),
            dirty: self.dirty.clone(),
            reanalyzed_hard_lines: self.reanalyzed_hard_lines.clone(),
            length_delta: self.length_delta,
            previous_byte_len: self.previous_byte_len,
            byte_len: self.byte_len,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextDocumentReceiptProjectionError {
    DocumentOwnerChanged,
    NonConsecutiveRevision,
    ByteRangeOverflow,
    InconsistentLengthDelta,
    InconsistentDocumentLength,
    ChangedRangeOutOfBounds,
    SelectionOutOfBounds,
    DocumentLengthOverflow,
    InvalidPublicReceipt(UiTextEditReceiptError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TextDocumentEditOutcome {
    /// The requested source range already contained the replacement bytes; nothing was invalidated.
    Unchanged { key: TextDocumentKey },
    /// Source changed and the receipt binds the old and newly published revisions.
    Changed(TextDocumentEditReceipt),
}

impl TextDocumentEditOutcome {
    pub(crate) fn revision(&self) -> UiTextDocumentRevision {
        let revision = match self {
            Self::Unchanged { key } => key.revision(),
            Self::Changed(receipt) => receipt.key.revision(),
        };
        UiTextDocumentRevision::new(revision)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextDocumentEditError {
    DocumentIdentityMismatch {
        expected: UiTextDocumentId,
        actual: UiTextDocumentId,
    },
    StaleDocument {
        expected: TextDocumentKey,
        actual: TextDocumentKey,
    },
    InvalidRange,
    InvalidUtf8Boundary,
    InvalidGraphemeBoundary,
    RevisionExhausted,
    LengthOverflow,
    StorageInvariant,
    HardLineInvariant,
}

impl TextDocument {
    /// Prepares a UTF-8-aligned replacement without publishing a revision or mutating storage.
    pub(crate) fn prepare_replace(
        &self,
        expected_key: TextDocumentKey,
        range: Range<usize>,
        replacement: &str,
    ) -> Result<PreparedTextDocumentReplace, TextDocumentEditError> {
        let actual_key = self.key();
        if expected_key != actual_key {
            return Err(TextDocumentEditError::StaleDocument {
                expected: expected_key,
                actual: actual_key,
            });
        }
        self.validate_range(&range)?;
        if self.range_equals(&range, replacement)? {
            return Ok(PreparedTextDocumentReplace::Unchanged {
                document_id: self.document_id,
                key: actual_key,
            });
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or(TextDocumentEditError::RevisionExhausted)?;
        let removed_len = range.len();
        let replacement_len = replacement.len();
        let previous_byte_len = self.byte_len;
        let next_byte_len = self
            .byte_len
            .checked_sub(removed_len)
            .and_then(|retained| retained.checked_add(replacement_len))
            .ok_or(TextDocumentEditError::LengthOverflow)?;
        let source_index_edit =
            self.source_index
                .prepare_incremental_edit(self, &range, replacement);
        let hard_line_edit =
            self.prepare_hard_line_edit(range.clone(), replacement, next_revision)?;
        let reanalyzed_hard_lines = hard_line_edit.receipt().clone();
        let previous_key = self.key();
        let (mut prefix, _) = self.pieces_split_at(range.start);
        let (_, suffix) = self.pieces_split_at(range.end);
        let addition_start = self.additions.len();
        let addition_end = addition_start
            .checked_add(replacement.len())
            .ok_or(TextDocumentEditError::LengthOverflow)?;
        let replacement = (!replacement.is_empty()).then(|| replacement.to_owned());
        if replacement.is_some() {
            let piece = TextDocumentPiece {
                source: TextDocumentPieceSource::Addition,
                range: addition_start..addition_end,
            };
            prefix.push(piece);
        }
        prefix.extend(suffix);
        Self::coalesce_pieces(&mut prefix);

        Ok(PreparedTextDocumentReplace::Changed(
            PreparedTextDocumentChange {
                document_id: self.document_id,
                previous_key,
                next_revision,
                dirty: TextDocumentDirtySpan {
                    old: range.start..range.end,
                    new: range.start..range.start + replacement_len,
                },
                reanalyzed_hard_lines,
                length_delta: length_delta(removed_len, replacement_len),
                previous_byte_len,
                byte_len: next_byte_len,
                replacement,
                pieces: prefix,
                hard_line_edit,
                source_index_edit,
            },
        ))
    }

    /// Commits a prepared replacement if the document still has its expected revision.
    pub(crate) fn commit_replace(
        &mut self,
        prepared: PreparedTextDocumentReplace,
    ) -> Result<TextDocumentEditOutcome, TextDocumentEditError> {
        let expected_document_id = prepared.document_id();
        if expected_document_id != self.document_id {
            return Err(TextDocumentEditError::DocumentIdentityMismatch {
                expected: expected_document_id,
                actual: self.document_id,
            });
        }
        let expected_key = prepared.expected_key();
        let actual_key = self.key();
        if expected_key != actual_key {
            return Err(TextDocumentEditError::StaleDocument {
                expected: expected_key,
                actual: actual_key,
            });
        }
        let PreparedTextDocumentReplace::Changed(prepared) = prepared else {
            return Ok(TextDocumentEditOutcome::Unchanged { key: actual_key });
        };

        Ok(TextDocumentEditOutcome::Changed(
            self.commit_prepared_change(prepared),
        ))
    }

    pub(super) fn commit_prepared_change(
        &mut self,
        prepared: PreparedTextDocumentChange,
    ) -> TextDocumentEditReceipt {
        self.flattened_snapshot = std::sync::OnceLock::new();
        if let Some(replacement) = prepared.replacement {
            self.additions.push_str(&replacement);
        }
        self.pieces = prepared.pieces;
        self.byte_len = prepared.byte_len;
        let source_index_edit = prepared.source_index_edit;
        let source_index_updated = source_index_edit.is_some_and(|edit| {
            self.source_index
                .apply_incremental_edit(prepared.next_revision, edit)
        });
        self.revision = prepared.next_revision;
        self.hard_line_models.apply(prepared.hard_line_edit);
        if !source_index_updated {
            self.source_index.invalidate();
        }

        TextDocumentEditReceipt {
            document_id: prepared.document_id,
            previous_key: prepared.previous_key,
            key: self.key(),
            dirty: prepared.dirty,
            reanalyzed_hard_lines: prepared.reanalyzed_hard_lines,
            length_delta: prepared.length_delta,
            previous_byte_len: prepared.previous_byte_len,
            byte_len: prepared.byte_len,
        }
    }

    /// Replaces a UTF-8-aligned source range without reconstructing unchanged chunks.
    ///
    /// Grapheme validation remains an input-policy concern until P1-19 migrates UI edit actions
    /// to document-boundary handles. This compatibility entry preserves one prepare/commit owner.
    pub(crate) fn replace(
        &mut self,
        expected_key: TextDocumentKey,
        range: Range<usize>,
        replacement: &str,
    ) -> Result<TextDocumentEditOutcome, TextDocumentEditError> {
        let prepared = self.prepare_replace(expected_key, range, replacement)?;
        self.commit_replace(prepared)
    }
}

fn length_delta(removed_len: usize, replacement_len: usize) -> TextDocumentLengthDelta {
    match replacement_len.cmp(&removed_len) {
        std::cmp::Ordering::Less => {
            TextDocumentLengthDelta::Decreased(removed_len - replacement_len)
        }
        std::cmp::Ordering::Equal => TextDocumentLengthDelta::Unchanged,
        std::cmp::Ordering::Greater => {
            TextDocumentLengthDelta::Increased(replacement_len - removed_len)
        }
    }
}

fn project_byte_range(
    range: &Range<usize>,
) -> Result<UiTextByteRange, TextDocumentReceiptProjectionError> {
    let start_byte = u32::try_from(range.start)
        .map_err(|_| TextDocumentReceiptProjectionError::ByteRangeOverflow)?;
    let end_byte = u32::try_from(range.end)
        .map_err(|_| TextDocumentReceiptProjectionError::ByteRangeOverflow)?;
    Ok(UiTextByteRange::new(start_byte, end_byte))
}
