use crate::text::document::TextDocumentStoreLimits;

const MVP_MAX_DOCUMENTS: usize = 1_024;
const MVP_MAX_DOCUMENT_BYTES: usize = 16 * 1024 * 1024;
const MVP_MAX_TOTAL_DOCUMENT_BYTES: usize = 64 * 1024 * 1024;
const MVP_MAX_REPLACEMENT_BYTES: usize = 4 * 1024 * 1024;
const MVP_MAX_RETAINED_SOURCE_BYTES_PER_DOCUMENT: usize = 64 * 1024 * 1024;
const MVP_MAX_TOTAL_RETAINED_SOURCE_BYTES: usize = 256 * 1024 * 1024;
const MVP_MAX_ADDITION_SOURCES_PER_DOCUMENT: usize = 1;
const MVP_MAX_PIECES_PER_DOCUMENT: usize = 1_048_576;
const MVP_MAX_CURRENT_SNAPSHOT_BYTES: usize = 64 * 1024 * 1024;
const MVP_MAX_ACTIVE_SNAPSHOT_LEASES: usize = 64;
const MVP_MAX_ACTIVE_SNAPSHOT_LEASE_BYTES: usize = 64 * 1024 * 1024;

/// Initial fail-closed limits for the runtime UI editing session.
///
/// Product-load profiling may tune these values, but callers never receive an unbounded store.
pub(super) const fn mvp_text_document_store_limits() -> TextDocumentStoreLimits {
    TextDocumentStoreLimits {
        max_documents: MVP_MAX_DOCUMENTS,
        max_document_bytes: MVP_MAX_DOCUMENT_BYTES,
        max_total_document_bytes: MVP_MAX_TOTAL_DOCUMENT_BYTES,
        max_replacement_bytes: MVP_MAX_REPLACEMENT_BYTES,
        max_retained_source_bytes_per_document: MVP_MAX_RETAINED_SOURCE_BYTES_PER_DOCUMENT,
        max_total_retained_source_bytes: MVP_MAX_TOTAL_RETAINED_SOURCE_BYTES,
        max_addition_sources_per_document: MVP_MAX_ADDITION_SOURCES_PER_DOCUMENT,
        max_pieces_per_document: MVP_MAX_PIECES_PER_DOCUMENT,
        max_current_snapshot_bytes: MVP_MAX_CURRENT_SNAPSHOT_BYTES,
        max_active_snapshot_leases: MVP_MAX_ACTIVE_SNAPSHOT_LEASES,
        max_active_snapshot_lease_bytes: MVP_MAX_ACTIVE_SNAPSHOT_LEASE_BYTES,
    }
}
