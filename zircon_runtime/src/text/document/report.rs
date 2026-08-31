use std::{mem::size_of, sync::Arc};

use super::storage::{TextDocument, TextDocumentPiece};

/// Content-free retained-memory accounting for one document authority.
///
/// The estimate includes owned source bytes and vector capacities, but not allocator headers or
/// `Arc` control blocks, so it is explicitly a lower bound rather than an admission limit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextDocumentStorageReport {
    pub(crate) revision: u64,
    pub(crate) byte_len: usize,
    pub(crate) original_bytes: usize,
    pub(crate) addition_source_count: usize,
    pub(crate) addition_bytes: usize,
    pub(crate) piece_count: usize,
    pub(crate) piece_capacity_bytes: usize,
    pub(crate) hard_line_count: usize,
    pub(crate) hard_line_capacity_bytes: usize,
    pub(crate) grapheme_boundary_count: usize,
    pub(crate) grapheme_boundary_capacity_bytes: usize,
    pub(crate) has_flattened_snapshot: bool,
    pub(crate) flattened_snapshot_bytes: usize,
    pub(crate) estimated_retained_bytes_lower_bound: usize,
}

impl TextDocument {
    pub(crate) fn storage_report(&self) -> TextDocumentStorageReport {
        let addition_bytes = self.additions.len();
        let addition_capacity_bytes = self.additions.capacity();
        let piece_capacity_bytes = self
            .pieces
            .capacity()
            .saturating_mul(size_of::<TextDocumentPiece>());
        let hard_line_capacity_bytes = self.hard_line_models.estimated_heap_bytes();
        let grapheme_boundary_capacity_bytes = self.source_index.estimated_heap_bytes();
        let flattened_snapshot = self.flattened_snapshot.get();
        let flattened_snapshot_bytes = flattened_snapshot
            .filter(|snapshot| !Arc::ptr_eq(snapshot, &self.original))
            .map_or(0, |snapshot| snapshot.len());
        let estimated_retained_bytes_lower_bound = size_of::<TextDocument>()
            .saturating_add(self.original.len())
            .saturating_add(addition_capacity_bytes)
            .saturating_add(piece_capacity_bytes)
            .saturating_add(hard_line_capacity_bytes)
            .saturating_add(grapheme_boundary_capacity_bytes)
            .saturating_add(flattened_snapshot_bytes);

        TextDocumentStorageReport {
            revision: self.revision,
            byte_len: self.byte_len,
            original_bytes: self.original.len(),
            addition_source_count: usize::from(!self.additions.is_empty()),
            addition_bytes,
            piece_count: self.pieces.len(),
            piece_capacity_bytes,
            hard_line_count: self.hard_line_models.lines().len(),
            hard_line_capacity_bytes,
            grapheme_boundary_count: self.source_index.grapheme_boundaries().len(),
            grapheme_boundary_capacity_bytes,
            has_flattened_snapshot: flattened_snapshot.is_some(),
            flattened_snapshot_bytes,
            estimated_retained_bytes_lower_bound,
        }
    }
}
