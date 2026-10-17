//! 为每个文本编辑会话建立有限的文档、保留源、索引快照与租约预算。
//! 字节阈值按 UTF-8 存储量计数；文档、片段、附加源和租约数量分别使用对应计数上限。

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
/// 返回新会话创建文档存储时使用的初始预算值。
///
/// # 返回与约束
/// 返回独立的限制配置；本函数不创建文档、不预留内存，也不按字符数换算字节阈值。
/// 预算拒绝由后续存储操作返回，并通过会话错误继续传递。
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
