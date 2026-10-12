use crate::core::editor_message::DocumentId;

use super::{
    CommandBox, CommandEffect, DetachedTransactionEventSink, EditCommand, EditCommandError,
    EditContext, EditWorldRoute, HistoryContextId, HistoryDetailPage, HistoryPageCursor,
    HistoryStatus, HistoryStore, MergeOutcome, SelectionSnapshot, TransactionEvent,
    TransactionEventDelivery, TransactionEventKind, TransactionEventSink, TransactionId,
    TransactionJournal, TransactionJournalError, TransactionRecord,
};

#[path = "transaction/dirty_batch.rs"]
mod dirty_batch;
#[path = "transaction/engine_state.rs"]
mod engine_state;
#[path = "transaction/exclusive_transition.rs"]
mod exclusive_transition;
#[path = "transaction/lifecycle.rs"]
mod lifecycle;
#[path = "transaction/observation.rs"]
mod observation;
#[path = "transaction/operation_gate.rs"]
mod operation_gate;
#[path = "transaction/operation_group.rs"]
mod operation_group;
#[path = "transaction/replay.rs"]
mod replay;
#[path = "transaction/save_token.rs"]
mod save_token;
#[path = "transaction/scope.rs"]
mod scope;
#[path = "transaction/volatile.rs"]
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
#[path = "tests/transaction_perf_source_guards.rs"]
mod performance_source_guards;
