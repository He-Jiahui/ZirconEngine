use std::sync::Arc;

use super::{
    EditCommandError, EditorTransactionEngine, HistoryContextId, HistoryDetailPage,
    HistoryPageCursor, HistoryStatus, HistoryStore, TransactionEvent, TransactionEventKind,
    TransactionId, TransactionJournal, TransactionJournalError, MAX_HISTORY_DETAIL_PAGE_SIZE,
};

impl EditorTransactionEngine {
    pub fn undo(&self, history: HistoryContextId) -> Result<bool, EditCommandError> {
        self.flush_operation_group()?;
        self.replay(history, true)
    }

    pub fn redo(&self, history: HistoryContextId) -> Result<bool, EditCommandError> {
        self.flush_operation_group()?;
        self.replay(history, false)
    }

    pub fn is_dirty(&self, history: HistoryContextId) -> Result<bool, EditCommandError> {
        self.flush_operation_group()?;
        self.start_operation("query dirty state")?;
        let mut state = self.lock_state();
        let dirty = !history.is_volatile()
            && state
                .histories
                .get(&history)
                .is_some_and(HistoryStore::is_dirty);
        self.clear_operation_locked(&mut state);
        Ok(dirty)
    }

    pub fn history_status(
        &self,
        history: HistoryContextId,
    ) -> Result<HistoryStatus, EditCommandError> {
        self.flush_operation_group()?;
        self.start_operation("query history status")?;
        let mut state = self.lock_state();
        let generation = Self::history_generation(&state, history);
        let status = match state.histories.get(&history) {
            Some(store) => store.status(generation),
            None => HistoryStatus::empty(generation),
        }
        .for_context(history);
        self.clear_operation_locked(&mut state);
        Ok(status)
    }

    pub fn history_details(
        &self,
        history: HistoryContextId,
        cursor: Option<&HistoryPageCursor>,
        page_size: usize,
    ) -> Result<HistoryDetailPage, EditCommandError> {
        if page_size == 0 || page_size > MAX_HISTORY_DETAIL_PAGE_SIZE {
            return Err(EditCommandError::HistoryPageSizeOutOfRange {
                requested: page_size,
                maximum: MAX_HISTORY_DETAIL_PAGE_SIZE,
            });
        }
        if cursor.is_some_and(|cursor| !cursor.belongs_to(&self.save_token_lineage)) {
            return Err(EditCommandError::HistoryPageCursorEngineMismatch);
        }
        if let Some(cursor) = cursor.filter(|cursor| cursor.history() != history) {
            return Err(EditCommandError::HistoryPageCursorHistoryMismatch {
                cursor_history: cursor.history(),
                requested_history: history,
            });
        }

        self.flush_operation_group()?;
        self.start_operation("query history details")?;
        let mut state = self.lock_state();
        let generation = Self::history_generation(&state, history);
        if let Some(cursor) = cursor.filter(|cursor| cursor.generation() != generation) {
            self.clear_operation_locked(&mut state);
            return Err(EditCommandError::HistoryPageCursorStale {
                history,
                cursor_generation: cursor.generation(),
                current_generation: generation,
            });
        }
        let offset = cursor.map_or(0, HistoryPageCursor::offset);
        let (status, records, has_more) = match state.histories.get(&history) {
            Some(store) => {
                let status = store.status(generation).for_context(history);
                let (records, has_more) = store.detail_window(offset, page_size);
                (status, records, has_more)
            }
            None => (
                HistoryStatus::empty(generation).for_context(history),
                Vec::new(),
                false,
            ),
        };
        let next_cursor = has_more.then(|| {
            HistoryPageCursor::new(
                Arc::clone(&self.save_token_lineage),
                history,
                generation,
                offset.saturating_add(records.len()),
            )
        });
        self.clear_operation_locked(&mut state);
        Ok(HistoryDetailPage::new(status, records, next_cursor))
    }

    pub fn journal_transaction(
        &self,
        history: HistoryContextId,
        transaction: TransactionId,
    ) -> Result<TransactionJournal, TransactionJournalError> {
        if history.is_volatile() {
            return Err(TransactionJournalError::VolatileHistory { history });
        }
        self.flush_operation_group()
            .map_err(TransactionJournalError::from)?;
        self.start_operation("serialize transaction journal")
            .map_err(TransactionJournalError::from)?;
        let mut state = self.lock_state();
        let journal = match state.histories.get(&history) {
            Some(store) => store.journal(history, transaction),
            None => Err(TransactionJournalError::TransactionNotFound {
                history,
                transaction,
            }),
        };
        self.clear_operation_locked(&mut state);
        journal
    }

    /// Returns the monotonic generation used to guard a save for one history context.
    pub fn history_generation_snapshot(
        &self,
        history: HistoryContextId,
    ) -> Result<u64, EditCommandError> {
        self.flush_operation_group()?;
        self.start_operation("snapshot history generation")?;
        let mut state = self.lock_state();
        let generation = Self::history_generation(&state, history);
        self.clear_operation_locked(&mut state);
        Ok(generation)
    }

    fn replay(&self, history: HistoryContextId, undo: bool) -> Result<bool, EditCommandError> {
        let operation = if undo { "undo" } else { "redo" };
        self.start_operation(operation)?;
        let (mut context, mut store, timestamp_frame, mutation) = {
            let mut state = self.lock_state();
            if !state.active.is_empty() {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::InvariantViolation {
                    invariant: "undo and redo require no active transaction scope",
                });
            }
            let can_replay = state.histories.get(&history).is_some_and(|store| {
                if undo {
                    store.can_undo()
                } else {
                    store.can_redo()
                }
            });
            if !can_replay {
                self.clear_operation_locked(&mut state);
                return Ok(false);
            }
            let mutation = match Self::reserve_history_mutation(&state, history) {
                Ok(reservation) => reservation,
                Err(error) => {
                    self.clear_operation_locked(&mut state);
                    return Err(error);
                }
            };
            let context = Self::take_context_from(&mut state)?;
            let Some(store) = state.histories.remove(&history) else {
                state.context = Some(context);
                self.clear_operation_locked(&mut state);
                return Ok(false);
            };
            (context, store, state.current_frame, mutation)
        };
        let route = match store.replay_route(undo).cloned() {
            Some(route) => route,
            None => {
                let mut state = self.lock_state();
                state.histories.insert(history, store);
                state.context = Some(context);
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::InvariantViolation {
                    invariant: "a replayable history must retain its target world route",
                });
            }
        };
        if let Err(error) = context.activate_world_route(&route) {
            let mut state = self.lock_state();
            state.histories.insert(history, store);
            state.context = Some(context);
            self.clear_operation_locked(&mut state);
            return Err(error);
        }
        let result = if undo {
            store.undo(context.as_mut())
        } else {
            store.redo(context.as_mut())
        };
        let faulted = matches!(&result, Err(EditCommandError::RollbackFailed { .. }));
        let event = result.as_ref().ok().and_then(|event_metadata| {
            event_metadata
                .as_ref()
                .map(|(transaction, label)| TransactionEvent {
                    transaction: *transaction,
                    history,
                    label: label.clone(),
                    timestamp_frame,
                    kind: if undo {
                        TransactionEventKind::UndoApplied
                    } else {
                        TransactionEventKind::RedoApplied
                    },
                })
        });
        let mut state = self.lock_state();
        state.histories.insert(history, store);
        if result.as_ref().ok().is_some_and(Option::is_some) {
            Self::record_history_mutation(&mut state, history, mutation);
        }
        state.context = Some(context);
        state.faulted |= faulted;
        self.clear_operation_locked(&mut state);
        drop(state);
        if let Some(event) = event {
            self.publish_event(event);
        }
        result.map(|record| record.is_some())
    }
}
