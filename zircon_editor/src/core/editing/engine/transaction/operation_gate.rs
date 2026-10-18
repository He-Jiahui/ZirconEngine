use std::sync::{Arc, MutexGuard};

use super::{EditCommandError, EditContext, EditorTransactionEngine, EngineState, TransactionId};

/// A close attempt owns mutation admission until capture tasks and teardown finish.
#[derive(Clone, Debug)]
pub(crate) struct SceneCloseAdmission {
    pub(super) identity: Arc<()>,
}

impl EditorTransactionEngine {
    pub(crate) fn validate_close_admission(
        &self,
        admission: &SceneCloseAdmission,
    ) -> Result<(), EditCommandError> {
        let state = self.lock_state();
        if !state
            .close_admission
            .upgrade()
            .is_some_and(|identity| Arc::ptr_eq(&identity, &admission.identity))
        {
            return Err(EditCommandError::EngineBusy {
                active: "a different main-window close attempt",
                requested: "capture closing document",
            });
        }
        Ok(())
    }

    pub(super) fn start_operation(&self, requested: &'static str) -> Result<(), EditCommandError> {
        self.start_admitted_operation(requested, None, false)
    }

    pub(super) fn start_read_operation(
        &self,
        requested: &'static str,
    ) -> Result<(), EditCommandError> {
        self.start_admitted_operation(requested, None, true)
    }

    pub(super) fn start_close_operation(
        &self,
        requested: &'static str,
        admission: &SceneCloseAdmission,
    ) -> Result<(), EditCommandError> {
        self.start_admitted_operation(requested, Some(admission), false)
    }

    fn start_admitted_operation(
        &self,
        requested: &'static str,
        admission: Option<&SceneCloseAdmission>,
        read_only: bool,
    ) -> Result<(), EditCommandError> {
        let mut state = self.lock_state();
        if let Some(identity) = state.close_admission.upgrade() {
            if !read_only
                && !admission.is_some_and(|admission| Arc::ptr_eq(&identity, &admission.identity))
            {
                return Err(EditCommandError::EngineBusy {
                    active: "main-window close",
                    requested,
                });
            }
        } else if admission.is_some() {
            return Err(EditCommandError::EngineBusy {
                active: "retired main-window close",
                requested,
            });
        }
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

    /// Restores client state while its owning reservation remains held.
    pub(super) fn restore_operation_context(&self, context: Box<dyn EditContext>, faulted: bool) {
        let mut state = self.lock_state();
        state.context = Some(context);
        state.faulted |= faulted;
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
