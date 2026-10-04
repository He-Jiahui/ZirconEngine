use super::{EditorTransactionEngine, EngineState, HistoryContextId, HistoryStatus};

mod decision;
pub use decision::HistoryDecisionToken;

impl EditorTransactionEngine {
    pub(super) fn start_observation(
        &self,
        requested: &'static str,
    ) -> Result<(), super::EditCommandError> {
        self.start_operation(requested)?;
        let mut state = self.lock_state();
        if let Some(group) = state
            .operation_group
            .as_ref()
            .filter(|group| !group.allows_observation())
        {
            let error = super::EditCommandError::EngineBusy {
                active: group.operation(),
                requested,
            };
            self.clear_operation_locked(&mut state);
            return Err(error);
        }
        Ok(())
    }

    /// Committed records keep their identity. Applied, uncommitted commands only affect
    /// the dirty/action projection; observing them never finalizes the interaction.
    pub(super) fn observed_history_status(
        state: &EngineState,
        history: HistoryContextId,
    ) -> HistoryStatus {
        let generation = Self::history_generation(state, history);
        let mut status = match state.histories.get(&history) {
            Some(store) => store.status(generation),
            None => HistoryStatus::empty(generation),
        };
        if state
            .active
            .iter()
            .any(|active| active.history == history && !active.commands.is_empty())
        {
            status.dirty = true;
            // Only a group has the explicit flush boundary used by undo/redo. A
            // manually owned scope must still be committed or cancelled by its owner.
            if state.operation_group.as_ref().is_some_and(|group| {
                state.active.iter().any(|active| {
                    active.history == history
                        && !active.commands.is_empty()
                        && group.transaction == Some(active.id)
                })
            }) {
                status.can_undo = true;
                status.can_redo = false;
            }
        }
        status.for_context(history)
    }

    /// Nested scopes are admitted only for the same history. A cursor remembers that
    /// one pending identity so cancellation can publish clean without a commit.
    pub(super) fn pending_dirty_history(state: &EngineState) -> Option<HistoryContextId> {
        state
            .active
            .iter()
            .find(|active| !active.history.is_volatile() && !active.commands.is_empty())
            .map(|active| active.history)
    }
}

#[cfg(test)]
#[path = "observation/tests/cases.rs"]
mod tests;
