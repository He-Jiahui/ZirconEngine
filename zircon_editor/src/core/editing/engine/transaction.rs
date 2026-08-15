use crate::core::editor_message::DocumentId;

use super::{
    CommandBox, CommandEffect, DetachedTransactionEventSink, EditCommand, EditCommandError,
    EditContext, HistoryContextId, HistoryDetailPage, HistoryPageCursor, HistoryStatus,
    HistoryStore, MergeOutcome, SelectionSnapshot, TransactionEvent, TransactionEventDelivery,
    TransactionEventKind, TransactionEventSink, TransactionId, TransactionJournal,
    TransactionJournalError, TransactionRecord,
};

mod dirty_batch;
mod engine_state;
mod exclusive_transition;
mod lifecycle;
mod operation_gate;
mod operation_group;
mod replay;
mod save_token;
mod scope;

pub use dirty_batch::{
    HistoryDirtyBatch, HistoryDirtyBatchKind, HistoryDirtyCursor, HistoryDirtyState,
};
pub use engine_state::{EditorTransactionEngine, MergeMode};
pub(crate) use exclusive_transition::ExclusiveTransition;
pub use operation_group::OperationTransactionResult;
pub use scope::TransactionScope;

use engine_state::{ActiveTransaction, EngineState};
use operation_group::{ActiveOperationGroup, OperationGroupReservation};

pub const MAX_HISTORY_DETAIL_PAGE_SIZE: usize = 128;

#[cfg(test)]
mod performance_source_guards {
    #[test]
    fn nested_cancel_does_not_remove_from_the_front_of_a_vec() {
        let source = include_str!("transaction.rs");
        let front_remove = ["frames", ".remove(0)"].concat();
        assert!(!source.contains(&front_remove));
    }

    #[test]
    fn transaction_root_remains_a_structural_facade() {
        let source = include_str!("transaction.rs");
        assert!(
            source.lines().count() <= 96,
            "transaction root exceeded facade budget"
        );
        for behavioral_item in [
            ["fn ", "begin_transaction"].concat(),
            ["fn ", "replay"].concat(),
            ["fn ", "commit_after_apply"].concat(),
            ["fn ", "cancel_frame"].concat(),
        ] {
            assert!(
                !source.contains(&behavioral_item),
                "transaction root retained behavioral item: {behavioral_item}"
            );
        }
    }
}
