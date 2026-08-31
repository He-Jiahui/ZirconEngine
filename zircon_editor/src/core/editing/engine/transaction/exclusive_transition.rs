use std::marker::PhantomData;
use std::rc::Rc;

use super::{EditCommandError, EditContext, EditorTransactionEngine, HistoryContextId};

/// Holds the engine's exclusive operation while a project-level context transition commits.
pub(crate) struct ExclusiveTransition<'engine> {
    pub(super) engine: &'engine EditorTransactionEngine,
    pub(super) not_send: PhantomData<Rc<()>>,
}

impl ExclusiveTransition<'_> {
    pub(crate) fn clear_history_and_context<T: 'static>(
        &mut self,
        history: HistoryContextId,
        expected_context: &'static str,
        update: impl FnOnce(&mut T) -> Result<(), EditCommandError>,
    ) -> Result<bool, EditCommandError> {
        let (mut context, mutation, stored_route) = {
            let mut state = self.engine.lock_state();
            let context = EditorTransactionEngine::take_context_from(&mut state)?;
            if !context.as_any().is::<T>() {
                state.context = Some(context);
                return Err(EditCommandError::ContextTypeMismatch {
                    expected: expected_context,
                });
            }
            let mutation = match EditorTransactionEngine::reserve_history_mutation(&state, history)
            {
                Ok(reservation) => reservation,
                Err(error) => {
                    state.context = Some(context);
                    return Err(error);
                }
            };
            let stored_route = match state
                .histories
                .get(&history)
                .map(|store| store.world_route())
            {
                Some(Ok(route)) => route.cloned(),
                Some(Err(error)) => {
                    state.context = Some(context);
                    return Err(error);
                }
                None => None,
            };
            (context, mutation, stored_route)
        };
        let route = match stored_route {
            Some(route) => route,
            None => match context.capture_world_route(history.world_domain()) {
                Ok(route) => route,
                Err(error) => {
                    self.restore_context(context, false);
                    return Err(error);
                }
            },
        };
        if let Err(error) = context.activate_world_route(&route) {
            self.restore_context(context, false);
            return Err(error);
        }
        let selection_before = context.selection_snapshot();
        let Some(typed_context) = context.as_any_mut().downcast_mut::<T>() else {
            self.restore_context(context, false);
            return Err(EditCommandError::ContextTypeMismatch {
                expected: expected_context,
            });
        };
        if let Err(command_error) = update(typed_context) {
            let restore_result = context.restore_selection(&selection_before);
            self.restore_context(context, restore_result.is_err());
            return match restore_result {
                Ok(()) => Err(command_error),
                Err(rollback_error) => Err(EditCommandError::RollbackFailed {
                    command_error: Box::new(command_error),
                    rollback_error: Box::new(rollback_error),
                }),
            };
        }

        let mut removed = {
            let mut state = self.engine.lock_state();
            let removed = state
                .histories
                .remove(&history)
                .map(|mut store| store.clear())
                .unwrap_or_default();
            EditorTransactionEngine::record_history_mutation(&mut state, history, mutation);
            removed
        };
        let changed = !removed.is_empty();
        for record in &mut removed {
            record.finalize(context.as_mut());
        }
        self.restore_context(context, false);
        Ok(changed)
    }

    fn restore_context(&self, context: Box<dyn EditContext>, faulted: bool) {
        let mut state = self.engine.lock_state();
        state.context = Some(context);
        state.faulted |= faulted;
    }
}

impl Drop for ExclusiveTransition<'_> {
    fn drop(&mut self) {
        self.engine.clear_operation();
    }
}
