use std::sync::Arc;

use crate::core::editing::engine::{
    EditCommandError, EditorTransactionEngine, HistoryContextId, HistoryDetailPage, TransactionId,
    MAX_HISTORY_DETAIL_PAGE_SIZE,
};

/// A bounded, read-only projection of one authoritative transaction history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionHistorySnapshot {
    pub context: HistoryContextId,
    pub generation: u64,
    pub total_count: usize,
    pub rows: Arc<[TransactionHistoryRowSnapshot]>,
    pub truncated: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub dirty: bool,
    pub saved_top_reachable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransactionHistoryRowSnapshot {
    pub transaction_id: TransactionId,
    pub label: String,
    pub timestamp_frame: u64,
    pub command_count: usize,
    pub participant_count: usize,
    pub significant: bool,
    pub applied: bool,
    pub is_top: bool,
    pub is_saved_top: bool,
}

impl TransactionHistorySnapshot {
    pub fn query(
        transactions: &EditorTransactionEngine,
        context: HistoryContextId,
    ) -> Result<Self, EditCommandError> {
        let page = transactions.history_details(context, None, MAX_HISTORY_DETAIL_PAGE_SIZE)?;
        Ok(Self::from_page(context, page))
    }

    pub(crate) fn from_page(context: HistoryContextId, page: HistoryDetailPage) -> Self {
        let status = page.status();
        let records = page.into_records();
        let truncated = status.len > records.len();
        let visible_top_index = status
            .top
            .and_then(|top| records.iter().position(|record| record.id == top));
        let top_is_after_visible_page =
            truncated && status.top.is_some() && visible_top_index.is_none();
        let rows = records
            .into_iter()
            .enumerate()
            .map(|(index, record)| TransactionHistoryRowSnapshot {
                transaction_id: record.id,
                label: record.label,
                timestamp_frame: record.timestamp_frame,
                command_count: record.command_count,
                participant_count: record.participants.len(),
                significant: record.significant,
                applied: top_is_after_visible_page
                    || visible_top_index.is_some_and(|top_index| index <= top_index),
                is_top: status.top == Some(record.id),
                is_saved_top: status.saved_top == Some(record.id),
            })
            .collect::<Vec<_>>()
            .into();

        Self {
            context,
            generation: status.generation,
            total_count: status.len,
            rows,
            truncated,
            can_undo: status.can_undo,
            can_redo: status.can_redo,
            dirty: status.dirty,
            saved_top_reachable: status.saved_top_reachable,
        }
    }
}
