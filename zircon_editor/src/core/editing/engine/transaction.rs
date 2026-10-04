use crate::core::editor_message::DocumentId;

use super::{
    CommandBox, CommandEffect, DetachedTransactionEventSink, EditCommand, EditCommandError,
    EditContext, EditWorldRoute, HistoryContextId, HistoryDetailPage, HistoryPageCursor,
    HistoryStatus, HistoryStore, MergeOutcome, SelectionSnapshot, TransactionEvent,
    TransactionEventDelivery, TransactionEventKind, TransactionEventSink, TransactionId,
    TransactionJournal, TransactionJournalError, TransactionRecord,
};

mod dirty_batch;
mod engine_state;
mod exclusive_transition;
mod lifecycle;
mod observation;
mod operation_gate;
mod operation_group;
mod replay;
mod save_token;
mod scope;
mod volatile;

pub use dirty_batch::{
    HistoryDirtyBatch, HistoryDirtyBatchKind, HistoryDirtyCursor, HistoryDirtyState,
};
pub use engine_state::{EditorTransactionEngine, MergeMode};
pub(crate) use exclusive_transition::ExclusiveTransition;
pub use observation::HistoryDecisionToken;
pub use operation_group::OperationTransactionResult;
pub use scope::TransactionScope;

use engine_state::{ActiveTransaction, EngineState};
use operation_group::{ActiveOperationGroup, OperationGroupReservation};

pub const MAX_HISTORY_DETAIL_PAGE_SIZE: usize = 128;

#[cfg(test)]
#[path = "tests/transaction_performance_source_guards.rs"]
mod performance_source_guards;
