mod edit;
mod hard_line_model;
mod index;
mod index_profile;
mod report;
mod storage;
mod store;

#[cfg(test)]
mod store_tests;
#[cfg(test)]
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
