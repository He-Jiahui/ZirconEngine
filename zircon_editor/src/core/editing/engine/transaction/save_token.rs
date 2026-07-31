use std::sync::Arc;

use crate::core::editing::engine::{
    EditCommandError, HistoryContextId, HistorySaveMarkOutcome, HistorySaveToken, HistoryStore,
};

use super::{EditorTransactionEngine, EngineState};

impl EditorTransactionEngine {
    pub fn capture_save_token(
        &self,
        history: HistoryContextId,
    ) -> Result<HistorySaveToken, EditCommandError> {
        self.flush_operation_group()?;
        self.start_operation("capture save token")?;
        let mut state = self.lock_state();
        if let Some(active) = state.active.last() {
            let error = EditCommandError::SaveTokenActiveTransaction {
                operation: "capture save token",
                active_history: active.history,
                transaction: active.id,
            };
            self.clear_operation_locked(&mut state);
            return Err(error);
        }
        let token = HistorySaveToken::new(
            Arc::clone(&self.save_token_lineage),
            history,
            state
                .histories
                .get(&history)
                .and_then(HistoryStore::current_transaction),
            Self::history_generation(&state, history),
        );
        self.clear_operation_locked(&mut state);
        Ok(token)
    }

    pub fn mark_saved_if_unchanged(
        &self,
        history: HistoryContextId,
        token: HistorySaveToken,
    ) -> Result<HistorySaveMarkOutcome, EditCommandError> {
        if !token.belongs_to(&self.save_token_lineage) {
            return Err(EditCommandError::SaveTokenEngineMismatch);
        }
        if token.history() != history {
            return Err(EditCommandError::SaveTokenHistoryMismatch {
                token_history: token.history(),
                requested_history: history,
            });
        }
        self.flush_operation_group()?;
        self.start_operation("mark saved if unchanged")?;
        let mut state = self.lock_state();
        if let Some(active) = state.active.last() {
            let error = EditCommandError::SaveTokenActiveTransaction {
                operation: "mark saved if unchanged",
                active_history: active.history,
                transaction: active.id,
            };
            self.clear_operation_locked(&mut state);
            return Err(error);
        }

        let current_generation = Self::history_generation(&state, history);
        let current_transaction = state
            .histories
            .get(&history)
            .and_then(HistoryStore::current_transaction);
        if token.generation() != current_generation || token.transaction() != current_transaction {
            let error = EditCommandError::HistoryChangedDuringSave {
                history,
                expected_generation: token.generation(),
                current_generation,
                expected_transaction: token.transaction(),
                current_transaction,
            };
            self.clear_operation_locked(&mut state);
            return Err(error);
        }

        let should_mark = state
            .histories
            .get(&history)
            .is_some_and(HistoryStore::is_dirty);
        let outcome = if should_mark {
            let dirty_change = match Self::reserve_dirty_change(&state) {
                Ok(reservation) => reservation,
                Err(error) => {
                    self.clear_operation_locked(&mut state);
                    return Err(error);
                }
            };
            state
                .histories
                .get_mut(&history)
                .expect("dirty history remains present while the engine lock is held")
                .mark_saved_current();
            Self::record_dirty_change(&mut state, history, dirty_change);
            HistorySaveMarkOutcome::Marked
        } else {
            HistorySaveMarkOutcome::AlreadyMarked
        };
        self.clear_operation_locked(&mut state);
        Ok(outcome)
    }

    pub(super) fn history_generation(state: &EngineState, history: HistoryContextId) -> u64 {
        state
            .history_generations
            .get(&history)
            .copied()
            .unwrap_or(0)
    }

    pub(super) fn next_history_generation(
        state: &EngineState,
        history: HistoryContextId,
    ) -> Result<u64, EditCommandError> {
        Self::history_generation(state, history)
            .checked_add(1)
            .ok_or(EditCommandError::HistoryGenerationExhausted { history })
    }

    #[cfg(test)]
    pub(crate) fn set_history_generation_for_test(
        &self,
        history: HistoryContextId,
        generation: u64,
    ) {
        self.lock_state()
            .history_generations
            .insert(history, generation);
    }
}
