mod command;
mod events;
mod history;
mod journal;
mod routing;
mod transaction;

pub use crate::core::editing::selection::{SelectionJournal, SelectionSnapshot};
pub use command::{
    CommandBox, CommandEffect, CommandExecutionError, EditCommand, EditCommandError, EditContext,
    MergeOutcome,
};
pub(crate) use events::DetachedTransactionEventSink;
pub use events::{
    TransactionEvent, TransactionEventDelivery, TransactionEventKind, TransactionEventSink,
};
pub use history::{
    HistoryContextId, HistoryDetailPage, HistoryPageCursor, HistoryRecordDetail,
    HistorySaveMarkOutcome, HistorySaveToken, HistoryStatus, HistoryStore, TransactionId,
    TransactionRecord,
};
pub use journal::{
    CommandJournalPayload, CommandJournalUnavailable, TransactionJournal, TransactionJournalError,
    TransactionJournalReadError, TransactionJournalSchemaError, TRANSACTION_JOURNAL_SCHEMA_VERSION,
};
pub use routing::resolve_history_context;
pub(crate) use transaction::ExclusiveTransition;
pub use transaction::{
    EditorTransactionEngine, HistoryDirtyBatch, HistoryDirtyBatchKind, HistoryDirtyCursor,
    HistoryDirtyState, MergeMode, OperationTransactionResult, TransactionScope,
    MAX_HISTORY_DETAIL_PAGE_SIZE,
};
