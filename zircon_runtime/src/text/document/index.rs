use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use super::{TextDocumentEditError, index_profile, storage::TextDocument};

/// Cached source-space boundaries for one document revision.
///
/// Hard-line identity belongs to the retained separator-aware model. This index owns only
/// revision-qualified grapheme boundaries needed by edit/navigation consumers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TextDocumentSourceIndex {
    valid: bool,
    revision: u64,
    grapheme_boundaries: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PreparedTextDocumentSourceIndexEdit {
    old_range: Range<usize>,
    replacement_len: usize,
    start_index: usize,
    end_index: usize,
}

impl TextDocumentSourceIndex {
    pub(super) fn empty() -> Self {
        Self {
            valid: false,
            revision: 0,
            grapheme_boundaries: Vec::new(),
        }
    }

    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn grapheme_boundaries(&self) -> &[usize] {
        &self.grapheme_boundaries
    }

    pub(super) fn matches_revision(&self, revision: u64) -> bool {
        self.valid && self.revision == revision
    }

    pub(super) fn invalidate(&mut self) {
        self.valid = false;
        self.grapheme_boundaries.clear();
    }

    pub(super) fn estimated_heap_bytes(&self) -> usize {
        self.grapheme_boundaries
            .capacity()
            .saturating_mul(std::mem::size_of::<usize>())
    }

    fn build(revision: u64, source: &str) -> Self {
        let mut grapheme_boundaries = Vec::new();
        grapheme_boundaries.push(0);
        grapheme_boundaries.extend(
            source
                .grapheme_indices(true)
                .map(|(start, grapheme)| start + grapheme.len()),
        );
        Self {
            valid: true,
            revision,
            grapheme_boundaries,
        }
    }

    pub(super) fn prepare_incremental_edit(
        &self,
        document: &TextDocument,
        old_range: &Range<usize>,
        replacement: &str,
    ) -> Option<PreparedTextDocumentSourceIndexEdit> {
        if !self.matches_revision(document.revision)
            || old_range.start > old_range.end
            || old_range.end > document.byte_len
        {
            return None;
        }
        let start_index = self
            .grapheme_boundaries
            .binary_search(&old_range.start)
            .ok()?;
        let end_index = self
            .grapheme_boundaries
            .binary_search(&old_range.end)
            .ok()?;
        let last_index = self.grapheme_boundaries.len().checked_sub(1)?;
        let next_index = end_index.checked_add(1)?.min(last_index);
        let context_start = self.grapheme_boundaries[start_index.saturating_sub(1)];
        let context_end = self.grapheme_boundaries[next_index];
        if !document.range_is_ascii_grapheme_edit(context_start..context_end)
            || !ascii_grapheme_edit(replacement.as_bytes())
        {
            return None;
        }
        Some(PreparedTextDocumentSourceIndexEdit {
            old_range: old_range.clone(),
            replacement_len: replacement.len(),
            start_index,
            end_index,
        })
    }

    pub(super) fn apply_incremental_edit(
        &mut self,
        next_revision: u64,
        edit: PreparedTextDocumentSourceIndexEdit,
    ) -> bool {
        let Some(previous_revision) = next_revision.checked_sub(1) else {
            return false;
        };
        if !self.matches_revision(previous_revision) {
            return false;
        }
        let started = index_profile::start_incremental_update();
        if self.grapheme_boundaries.get(edit.start_index).copied() != Some(edit.old_range.start)
            || self.grapheme_boundaries.get(edit.end_index).copied() != Some(edit.old_range.end)
            || edit.start_index > edit.end_index
        {
            return false;
        }
        let start_index = edit.start_index;
        let end_index = edit.end_index;
        let Some(old_len) = edit.old_range.end.checked_sub(edit.old_range.start) else {
            return false;
        };
        let new_len = edit.replacement_len;
        let delta = if new_len >= old_len {
            new_len - old_len
        } else {
            old_len - new_len
        };
        let new_document_len = if new_len >= old_len {
            self.grapheme_boundaries
                .last()
                .copied()
                .and_then(|length| length.checked_add(delta))
        } else {
            self.grapheme_boundaries
                .last()
                .copied()
                .and_then(|length| length.checked_sub(delta))
        };
        let Some(new_document_len) = new_document_len else {
            return false;
        };
        let Some(removed_boundary_count) = end_index.checked_sub(start_index) else {
            return false;
        };
        let Some(retained_boundary_count) = self
            .grapheme_boundaries
            .len()
            .checked_sub(removed_boundary_count)
        else {
            return false;
        };
        let Some(capacity) = retained_boundary_count.checked_add(new_len) else {
            return false;
        };
        let Some(replacement_end) = edit.old_range.start.checked_add(new_len) else {
            return false;
        };
        let Some(suffix_start) = end_index.checked_add(1) else {
            return false;
        };
        let mut boundaries = Vec::with_capacity(capacity);
        boundaries.extend_from_slice(&self.grapheme_boundaries[..=start_index]);
        if new_len > 0 {
            let Some(first_replacement_boundary) = edit.old_range.start.checked_add(1) else {
                return false;
            };
            boundaries.extend(first_replacement_boundary..=replacement_end);
        }
        for &boundary in &self.grapheme_boundaries[suffix_start..] {
            let shifted = if new_len >= old_len {
                boundary.checked_add(delta)
            } else {
                boundary.checked_sub(delta)
            };
            let Some(shifted) = shifted else {
                return false;
            };
            boundaries.push(shifted);
        }
        if boundaries.last().copied() != Some(new_document_len) {
            return false;
        }
        self.grapheme_boundaries = boundaries;
        self.revision = next_revision;
        index_profile::finish_incremental_update(
            old_len.saturating_add(new_len),
            self.grapheme_boundaries.len(),
            started,
        );
        true
    }
}

fn ascii_grapheme_edit(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .all(|&byte| byte.is_ascii() && byte != b'\r' && byte != b'\n')
}

impl TextDocument {
    pub(crate) fn source_index(&mut self) -> &TextDocumentSourceIndex {
        if self.source_index.matches_revision(self.revision) {
            index_profile::record_index_hit();
        } else {
            let started = index_profile::start_index_rebuild();
            let source = self.snapshot_lease();
            self.source_index = TextDocumentSourceIndex::build(self.revision, source.as_str());
            index_profile::finish_index_rebuild(
                source.as_str().len(),
                self.source_index.grapheme_boundaries.len(),
                started,
            );
        }
        &self.source_index
    }

    pub(crate) fn retained_grapheme_count(
        &mut self,
        replaced_range: Range<usize>,
    ) -> Result<usize, TextDocumentEditError> {
        self.validate_range(&replaced_range)?;
        let started = index_profile::start_query();
        let boundaries = self.source_index().grapheme_boundaries();
        let result = (|| {
            let start = boundaries.binary_search(&replaced_range.start);
            index_profile::record_binary_searches(1);
            let start = start.map_err(|_| TextDocumentEditError::InvalidGraphemeBoundary)?;
            let end = boundaries.binary_search(&replaced_range.end);
            index_profile::record_binary_searches(1);
            let end = end.map_err(|_| TextDocumentEditError::InvalidGraphemeBoundary)?;
            Ok(boundaries
                .len()
                .saturating_sub(1)
                .saturating_sub(end.saturating_sub(start)))
        })();
        index_profile::finish_query(started);
        result
    }
}
