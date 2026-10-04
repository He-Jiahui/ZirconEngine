use std::marker::PhantomData;
use std::rc::Rc;

use super::{
    EditCommandError, EditContext, EditorTransactionEngine, HistoryContextId, HistoryDecisionToken,
    HistoryStatus, TransactionEvent,
};

/// Holds the engine's exclusive operation while a project-level context transition commits.
pub(crate) struct ExclusiveTransition<'engine> {
    pub(super) engine: &'engine EditorTransactionEngine,
    pub(super) not_send: PhantomData<Rc<()>>,
    deferred_event: Option<TransactionEvent>,
}

impl EditorTransactionEngine {
    /// Starts admission before finalizing a group, retaining the gate through replacement.
    pub(crate) fn begin_exclusive_transition(
        &self,
        operation: &'static str,
    ) -> Result<ExclusiveTransition<'_>, EditCommandError> {
        self.start_observation(operation)?;
        let mut transition = ExclusiveTransition {
            engine: self,
            not_send: PhantomData,
            deferred_event: None,
        };
        transition.flush_pending_group()?;
        Ok(transition)
    }

    /// Atomically consumes the dirty decision before any pending group is finalized.
    /// Missing authorization admits only clean history; stale decisions preserve pending state.
    pub(crate) fn begin_history_transition<T: 'static>(
        &self,
        operation: &'static str,
        history: HistoryContextId,
        authorized: Option<&HistoryDecisionToken>,
        expected_context: &'static str,
        prepare: impl FnOnce(&T, &super::EditWorldRoute) -> Result<(), EditCommandError>,
    ) -> Result<ExclusiveTransition<'_>, EditCommandError> {
        self.start_observation(operation)?;
        let mut transition = ExclusiveTransition {
            engine: self,
            not_send: PhantomData,
            deferred_event: None,
        };
        let current = self.dirty_history_decision_reserved(history)?;
        if current.as_ref() != authorized {
            return Err(EditCommandError::HistoryDecisionChanged { history });
        }
        if let Some(token) = current.as_ref() {
            self.validate_history_decision_route_reserved(history, token)?;
        }
        transition.prepare_history_clear::<T>(history, expected_context, prepare)?;
        transition.flush_pending_group()?;
        Ok(transition)
    }
}

impl ExclusiveTransition<'_> {
    /// Resolves all deterministic preparation before committing an authorized interaction.
    /// Client context methods and the read-only preparation callback run outside the mutex.
    fn prepare_history_clear<T: 'static>(
        &self,
        history: HistoryContextId,
        expected_context: &'static str,
        prepare: impl FnOnce(&T, &super::EditWorldRoute) -> Result<(), EditCommandError>,
    ) -> Result<(), EditCommandError> {
        let (context, stored_route, pending_route) = {
            let mut state = self.engine.lock_state();
            let pending_commit = match state.operation_group.as_ref() {
                Some(group) => {
                    if !group.allows_observation() {
                        return Err(EditCommandError::EngineBusy {
                            active: group.operation(),
                            requested: "prepare history clear",
                        });
                    }
                    let transaction =
                        group
                            .transaction
                            .ok_or(EditCommandError::InvariantViolation {
                                invariant: "an open operation group must own a transaction",
                            })?;
                    if state.active.len() != 1 || state.active[0].id != transaction {
                        return Err(EditCommandError::InvariantViolation {
                            invariant: "exclusive group flush requires its sole root scope",
                        });
                    }
                    (!state.active[0].commands.is_empty()).then_some(group.history)
                }
                None if !state.active.is_empty() => {
                    return Err(EditCommandError::InvariantViolation {
                        invariant:
                            "exclusive editor transitions require no active transaction scope",
                    })
                }
                None => None,
            };
            EditorTransactionEngine::preflight_history_clear(&state, history, pending_commit)?;
            let stored_route = state
                .histories
                .get(&history)
                .map(|store| store.world_route())
                .transpose()?
                .flatten()
                .cloned();
            let pending_route = state.active.first().map(|active| active.route.clone());
            (
                EditorTransactionEngine::take_context_from(&mut state)?,
                stored_route,
                pending_route,
            )
        };
        let result = (|| {
            let typed = context.as_any().downcast_ref::<T>().ok_or(
                EditCommandError::ContextTypeMismatch {
                    expected: expected_context,
                },
            )?;
            let route = context.capture_world_route(history.world_domain())?;
            if stored_route.as_ref().is_some_and(|stored| stored != &route) {
                return Err(EditCommandError::WorldRouteStale {
                    world_domain: history.world_domain(),
                });
            }
            // Even a group belonging to a different history must retain its own current route.
            if let Some(pending) = pending_route.as_ref() {
                if context.capture_world_route(pending.world_domain())? != *pending {
                    return Err(EditCommandError::WorldRouteStale {
                        world_domain: pending.world_domain(),
                    });
                }
            }
            prepare(typed, &route)
        })();
        self.engine.restore_operation_context(context, false);
        result
    }

    /// Flushes only an engine-owned group, using the existing commit implementation.
    /// Manual scopes remain owned by their caller and are rejected without finalization.
    fn flush_pending_group(&mut self) -> Result<(), EditCommandError> {
        let transaction = {
            let mut state = self.engine.lock_state();
            let Some(group) = state.operation_group.as_ref() else {
                if !state.active.is_empty() {
                    return Err(EditCommandError::InvariantViolation {
                        invariant:
                            "exclusive editor transitions require no active transaction scope",
                    });
                }
                return Ok(());
            };
            if !group.allows_observation() {
                return Err(EditCommandError::EngineBusy {
                    active: group.operation(),
                    requested: "exclusive transition",
                });
            }
            let transaction = group
                .transaction
                .ok_or(EditCommandError::InvariantViolation {
                    invariant: "an open operation group must own a transaction",
                })?;
            if state.active.len() != 1 || state.active[0].id != transaction {
                return Err(EditCommandError::InvariantViolation {
                    invariant: "exclusive group flush requires its sole root scope",
                });
            }
            state
                .operation_group
                .as_mut()
                .expect("the reserved group remains present")
                .begin_flush("exclusive transition")?;
            transaction
        };
        let result = self.engine.commit_after_apply_reserved(
            transaction,
            &mut |_: &super::SelectionSnapshot| Ok(()),
            &mut self.deferred_event,
        );
        let preserve = result.is_err() && self.engine.has_active_scope(transaction);
        let mut state = self.engine.lock_state();
        if state
            .operation_group
            .as_ref()
            .is_some_and(|group| group.transaction == Some(transaction))
        {
            if preserve {
                state
                    .operation_group
                    .as_mut()
                    .expect("failed commit retains its group")
                    .restore_open_after_failed_flush();
            } else {
                state.operation_group = None;
            }
        }
        result.map(|_| ())
    }

    /// Observes history without releasing or reentering the exclusive operation.
    pub(crate) fn history_status(&self, history: HistoryContextId) -> HistoryStatus {
        let state = self.engine.lock_state();
        let generation = EditorTransactionEngine::history_generation(&state, history);
        match state.histories.get(&history) {
            Some(store) => store.status(generation),
            None => HistoryStatus::empty(generation),
        }
        .for_context(history)
    }

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
        // Notification callbacks may reenter the engine only after reservation release.
        if let Some(event) = self.deferred_event.take() {
            self.engine.publish_event(event);
        }
    }
}
