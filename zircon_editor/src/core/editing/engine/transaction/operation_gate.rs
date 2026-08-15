use std::sync::MutexGuard;

use super::{EditCommandError, EditContext, EditorTransactionEngine, EngineState, TransactionId};

impl EditorTransactionEngine {
    pub(super) fn start_operation(&self, requested: &'static str) -> Result<(), EditCommandError> {
        let mut state = self.lock_state();
        if state.faulted {
            return Err(EditCommandError::EngineFaulted {
                operation: requested,
            });
        }
        if let Some(active) = state.operation {
            return Err(EditCommandError::EngineBusy { active, requested });
        }
        state.operation = Some(requested);
        Ok(())
    }

    pub(super) fn clear_operation(&self) {
        let mut state = self.lock_state();
        self.clear_operation_locked(&mut state);
    }

    pub(super) fn clear_operation_locked(&self, state: &mut EngineState) {
        state.operation = None;
        self.operation_changed.notify_all();
    }

    pub(super) fn wait_for_operation(&self) {
        let mut state = self.lock_state();
        while state.operation.is_some() {
            state = match self.operation_changed.wait(state) {
                Ok(state) => state,
                Err(poison) => poison.into_inner(),
            };
        }
    }

    pub(super) fn take_context(&self) -> Result<Box<dyn EditContext>, EditCommandError> {
        Self::take_context_from(&mut self.lock_state())
    }

    pub(super) fn take_context_from(
        state: &mut EngineState,
    ) -> Result<Box<dyn EditContext>, EditCommandError> {
        state
            .context
            .take()
            .ok_or(EditCommandError::InvariantViolation {
                invariant: "an active engine operation owns the edit context",
            })
    }

    pub(super) fn finish_operation(&self, context: Box<dyn EditContext>, faulted: bool) {
        let mut state = self.lock_state();
        state.context = Some(context);
        state.faulted |= faulted;
        self.clear_operation_locked(&mut state);
    }

    pub(super) fn has_active_scope(&self, scope: TransactionId) -> bool {
        self.lock_state()
            .active
            .iter()
            .any(|active| active.id == scope)
    }

    pub(super) fn next_transaction_id(
        state: &mut EngineState,
    ) -> Result<TransactionId, EditCommandError> {
        let sequence = state.next_transaction;
        state.next_transaction = state
            .next_transaction
            .checked_add(1)
            .ok_or(EditCommandError::TransactionIdExhausted)?;
        Ok(TransactionId::from_sequence(sequence))
    }

    pub(super) fn lock_state(&self) -> MutexGuard<'_, EngineState> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poison) => poison.into_inner(),
        }
    }
}
