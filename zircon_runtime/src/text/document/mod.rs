//! 保留可编辑 UTF-8 源文档及其版本、硬行和字素索引；UI 会话经 Store 取得受预算约束的编辑与快照能力。
//! 排版、整形和绘制只消费已提交的源或回执，不在此模块持有视觉行状态。

mod edit;
mod hard_line_model;
mod index;
mod index_profile;
mod report;
mod storage;
mod store;

#[cfg(test)]
#[path = "tests/store_tests.rs"]
mod store_tests;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

pub(crate) use edit::{
    PreparedTextDocumentChange, PreparedTextDocumentReplace, TextDocumentDirtySpan,
    TextDocumentEditError, TextDocumentEditOutcome, TextDocumentEditReceipt,
    TextDocumentLengthDelta, TextDocumentReceiptProjectionError,
};
pub(crate) use hard_line_model::{
    TextDocumentHardLineId, TextDocumentHardLineModel, TextDocumentHardLineSpan,
};
pub(crate) use index::TextDocumentSourceIndex;
pub(crate) use report::TextDocumentStorageReport;
pub(crate) use storage::{TextDocument, TextDocumentSnapshotLease};
pub(crate) use store::{
    ManagedTextDocumentSnapshotLease, OpenedTextDocument, PreparedTextDocumentStoreEdit,
    TextDocumentAdmissionFailure, TextDocumentStore, TextDocumentStoreEditCommit,
    TextDocumentStoreError, TextDocumentStoreLimits, TextDocumentStoreReport,
};
