use std::{
    fmt,
    ops::Range,
    sync::{Arc, OnceLock},
};

use super::hard_line_model::TextDocumentHardLineModels;
use super::index::TextDocumentSourceIndex;
use crate::text::TextDocumentKey;
use zircon_runtime_interface::ui::text::{UiTextDocumentId, UiTextDocumentRevision};

/// Persistent UTF-8 source storage for one text owner.
///
/// The document keeps one immutable original source plus one append-only addition source and
/// references them through pieces. Editing changes the piece list and appends admitted replacement
/// bytes; a flattened `String` is produced exclusively by an explicit snapshot request for legacy
/// serialization or an unmigrated consumer.
pub(crate) struct TextDocument {
    pub(super) document_id: UiTextDocumentId,
    pub(super) owner: u64,
    pub(super) revision: u64,
    pub(super) original: Arc<str>,
    pub(super) additions: String,
    pub(super) pieces: Vec<TextDocumentPiece>,
    pub(super) byte_len: usize,
    pub(super) hard_line_models: TextDocumentHardLineModels,
    pub(super) source_index: TextDocumentSourceIndex,
    pub(super) flattened_snapshot: OnceLock<Arc<str>>,
}

impl fmt::Debug for TextDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TextDocument")
            .field("document_id", &self.document_id)
            .field("owner", &self.owner)
            .field("revision", &self.revision)
            .field("byte_len", &self.byte_len)
            .field(
                "addition_source_count",
                &usize::from(!self.additions.is_empty()),
            )
            .field("piece_count", &self.pieces.len())
            .field("hard_line_count", &self.hard_line_models.lines().len())
            .field(
                "has_flattened_snapshot",
                &self.flattened_snapshot.get().is_some(),
            )
            .finish()
    }
}

/// Immutable, revision-bound contiguous source for legacy and shaping consumers.
///
/// Cloning a lease only clones the `Arc`. The document flattens its piece storage at most once for
/// a revision and keeps older leased revisions alive independently.
#[derive(Clone)]
pub(crate) struct TextDocumentSnapshotLease {
    document_id: UiTextDocumentId,
    revision: UiTextDocumentRevision,
    key: TextDocumentKey,
    source: Arc<str>,
}

impl fmt::Debug for TextDocumentSnapshotLease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TextDocumentSnapshotLease")
            .field("document_id", &self.document_id)
            .field("revision", &self.revision)
            .field("key", &self.key)
            .field("byte_len", &self.source.len())
            .finish()
    }
}

impl TextDocumentSnapshotLease {
    pub(crate) const fn document_id(&self) -> UiTextDocumentId {
        self.document_id
    }

    pub(crate) const fn revision(&self) -> UiTextDocumentRevision {
        self.revision
    }

    pub(crate) const fn key(&self) -> TextDocumentKey {
        self.key
    }

    pub(crate) fn as_str(&self) -> &str {
        self.source.as_ref()
    }

    pub(crate) const fn shared_source(&self) -> &Arc<str> {
        &self.source
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TextDocumentPieceSource {
    Original,
    Addition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct TextDocumentPiece {
    pub(super) source: TextDocumentPieceSource,
    pub(super) range: Range<usize>,
}

impl TextDocument {
    /// Creates one retained document authority.
    ///
    /// `TextDocument` intentionally is not `Clone`: copying an owner and revision would let two
    /// mutable sources publish different content under the same `TextDocumentKey`.
    pub(crate) fn new(owner: u64, text: impl Into<Arc<str>>) -> Self {
        let original = text.into();
        let flattened_snapshot = OnceLock::from(Arc::clone(&original));
        let byte_len = original.len();
        let hard_line_models = TextDocumentHardLineModels::new(0, &original);
        let pieces = (!original.is_empty())
            .then_some(TextDocumentPiece {
                source: TextDocumentPieceSource::Original,
                range: 0..byte_len,
            })
            .into_iter()
            .collect();
        Self {
            document_id: UiTextDocumentId::issue(),
            owner,
            revision: 0,
            original,
            additions: String::new(),
            pieces,
            byte_len,
            hard_line_models,
            source_index: TextDocumentSourceIndex::empty(),
            flattened_snapshot,
        }
    }

    pub(crate) fn key(&self) -> TextDocumentKey {
        TextDocumentKey::new(self.owner, self.revision)
    }

    pub(crate) const fn document_id(&self) -> UiTextDocumentId {
        self.document_id
    }

    pub(crate) const fn len(&self) -> usize {
        self.byte_len
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.byte_len == 0
    }

    pub(crate) fn snapshot(&self) -> String {
        self.snapshot_lease().as_str().to_owned()
    }

    pub(crate) fn snapshot_lease(&self) -> TextDocumentSnapshotLease {
        let source = self
            .flattened_snapshot
            .get_or_init(|| Arc::from(self.snapshot_range_unchecked(0..self.byte_len)));
        TextDocumentSnapshotLease {
            document_id: self.document_id,
            revision: UiTextDocumentRevision::new(self.revision),
            key: self.key(),
            source: Arc::clone(source),
        }
    }

    pub(crate) fn snapshot_range(
        &self,
        range: Range<usize>,
    ) -> Result<String, super::TextDocumentEditError> {
        self.validate_range(&range)?;
        Ok(self.snapshot_range_unchecked(range))
    }

    pub(crate) fn original_chunk(&self) -> Arc<str> {
        Arc::clone(&self.original)
    }

    pub(crate) fn references_original_chunk(&self, original: &Arc<str>) -> bool {
        Arc::ptr_eq(&self.original, original)
            && self
                .pieces
                .iter()
                .any(|piece| matches!(piece.source, TextDocumentPieceSource::Original))
    }

    pub(super) fn validate_range(
        &self,
        range: &Range<usize>,
    ) -> Result<(), super::TextDocumentEditError> {
        if range.start > range.end || range.end > self.byte_len {
            return Err(super::TextDocumentEditError::InvalidRange);
        }
        if !self.is_utf8_boundary(range.start) || !self.is_utf8_boundary(range.end) {
            return Err(super::TextDocumentEditError::InvalidUtf8Boundary);
        }
        Ok(())
    }

    /// Compares a document range across piece boundaries without flattening it into a `String`.
    pub(super) fn range_equals(
        &self,
        range: &Range<usize>,
        candidate: &str,
    ) -> Result<bool, super::TextDocumentEditError> {
        if range.len() != candidate.len() {
            return Ok(false);
        }
        if candidate.is_empty() {
            return Ok(true);
        }

        let candidate = candidate.as_bytes();
        let mut candidate_offset = 0usize;
        let mut document_offset = 0usize;
        for piece in &self.pieces {
            let piece_len = piece.range.len();
            let piece_end = document_offset
                .checked_add(piece_len)
                .ok_or(super::TextDocumentEditError::LengthOverflow)?;
            let overlap_start = range.start.max(document_offset);
            let overlap_end = range.end.min(piece_end);
            if overlap_start < overlap_end {
                let overlap_len = overlap_end - overlap_start;
                let source_start = piece
                    .range
                    .start
                    .checked_add(overlap_start - document_offset)
                    .ok_or(super::TextDocumentEditError::LengthOverflow)?;
                let source_end = source_start
                    .checked_add(overlap_len)
                    .ok_or(super::TextDocumentEditError::LengthOverflow)?;
                let candidate_end = candidate_offset
                    .checked_add(overlap_len)
                    .ok_or(super::TextDocumentEditError::LengthOverflow)?;
                let source = self
                    .piece_source_bytes(piece)
                    .and_then(|source| source.get(source_start..source_end))
                    .ok_or(super::TextDocumentEditError::StorageInvariant)?;
                let expected = candidate
                    .get(candidate_offset..candidate_end)
                    .ok_or(super::TextDocumentEditError::StorageInvariant)?;
                if source != expected {
                    return Ok(false);
                }
                candidate_offset = candidate_end;
            }
            if piece_end >= range.end {
                break;
            }
            document_offset = piece_end;
        }

        if candidate_offset != candidate.len() {
            return Err(super::TextDocumentEditError::StorageInvariant);
        }
        Ok(true)
    }

    pub(super) fn pieces_split_at(
        &self,
        offset: usize,
    ) -> (Vec<TextDocumentPiece>, Vec<TextDocumentPiece>) {
        let mut before = Vec::with_capacity(self.pieces.len().saturating_add(1));
        let mut after = Vec::with_capacity(self.pieces.len().saturating_add(1));
        let mut document_offset = 0usize;

        for piece in &self.pieces {
            let piece_len = piece.range.len();
            let piece_end = document_offset.saturating_add(piece_len);
            if piece_end <= offset {
                before.push(piece.clone());
            } else if document_offset >= offset {
                after.push(piece.clone());
            } else {
                let split = offset - document_offset;
                if split > 0 {
                    before.push(TextDocumentPiece {
                        source: piece.source,
                        range: piece.range.start..piece.range.start + split,
                    });
                }
                if split < piece_len {
                    after.push(TextDocumentPiece {
                        source: piece.source,
                        range: piece.range.start + split..piece.range.end,
                    });
                }
            }
            document_offset = piece_end;
        }
        (before, after)
    }

    pub(super) fn coalesce_pieces(pieces: &mut Vec<TextDocumentPiece>) {
        let mut merged: Vec<TextDocumentPiece> = Vec::with_capacity(pieces.len());
        for piece in pieces.drain(..) {
            if let Some(previous) = merged.last_mut() {
                if previous.source == piece.source && previous.range.end == piece.range.start {
                    previous.range.end = piece.range.end;
                    continue;
                }
            }
            merged.push(piece);
        }
        *pieces = merged;
    }

    pub(super) fn snapshot_range_unchecked(&self, range: Range<usize>) -> String {
        let mut snapshot = String::with_capacity(range.len());
        let mut document_offset = 0usize;
        for piece in &self.pieces {
            let piece_len = piece.range.len();
            let piece_end = document_offset.saturating_add(piece_len);
            let start = range.start.max(document_offset);
            let end = range.end.min(piece_end);
            if start < end {
                let relative_start = piece.range.start + (start - document_offset);
                let relative_end = piece.range.start + (end - document_offset);
                snapshot.push_str(&self.piece_source(piece)[relative_start..relative_end]);
            }
            if piece_end >= range.end {
                break;
            }
            document_offset = piece_end;
        }
        snapshot
    }

    /// Checks a validated source window without materializing a temporary string.
    ///
    /// Incremental grapheme-index admission only needs to know whether the old context is
    /// byte-wise ASCII and free of hard-line separators. Walking the existing pieces keeps this
    /// preflight allocation-free; the caller still controls the window size through cached
    /// grapheme boundaries.
    pub(super) fn range_is_ascii_grapheme_edit(&self, range: Range<usize>) -> bool {
        if range.start > range.end || range.end > self.byte_len {
            return false;
        }
        if range.start == range.end {
            return true;
        }

        let mut document_offset = 0usize;
        for piece in &self.pieces {
            let Some(piece_end) = document_offset.checked_add(piece.range.len()) else {
                return false;
            };
            let overlap_start = range.start.max(document_offset);
            let overlap_end = range.end.min(piece_end);
            if overlap_start < overlap_end {
                let relative_start = overlap_start - document_offset;
                let relative_end = overlap_end - document_offset;
                let Some(source_start) = piece.range.start.checked_add(relative_start) else {
                    return false;
                };
                let Some(source_end) = piece.range.start.checked_add(relative_end) else {
                    return false;
                };
                let Some(bytes) = self
                    .piece_source_bytes(piece)
                    .and_then(|source| source.get(source_start..source_end))
                else {
                    return false;
                };
                if !bytes
                    .iter()
                    .all(|&byte| byte.is_ascii() && byte != b'\r' && byte != b'\n')
                {
                    return false;
                }
            }
            if piece_end >= range.end {
                return true;
            }
            document_offset = piece_end;
        }
        false
    }

    fn is_utf8_boundary(&self, offset: usize) -> bool {
        if offset > self.byte_len {
            return false;
        }
        if offset == 0 || offset == self.byte_len {
            return true;
        }

        let mut document_offset = 0usize;
        for piece in &self.pieces {
            let piece_end = document_offset.saturating_add(piece.range.len());
            if offset == document_offset || offset == piece_end {
                return true;
            }
            if offset < piece_end {
                return self
                    .piece_source(piece)
                    .is_char_boundary(piece.range.start + offset - document_offset);
            }
            document_offset = piece_end;
        }
        false
    }

    fn piece_source(&self, piece: &TextDocumentPiece) -> &str {
        match piece.source {
            TextDocumentPieceSource::Original => self.original.as_ref(),
            TextDocumentPieceSource::Addition => self.additions.as_str(),
        }
    }

    fn piece_source_bytes(&self, piece: &TextDocumentPiece) -> Option<&[u8]> {
        match piece.source {
            TextDocumentPieceSource::Original => Some(self.original.as_bytes()),
            TextDocumentPieceSource::Addition => Some(self.additions.as_bytes()),
        }
    }
}
