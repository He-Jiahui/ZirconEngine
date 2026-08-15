use std::marker::PhantomData;

use super::{
    ActiveTransaction, CommandBox, CommandEffect, EditCommand, EditCommandError, EditContext,
    EditorTransactionEngine, ExclusiveTransition, HistoryContextId, HistoryStore, MergeMode,
    MergeOutcome, SelectionSnapshot, TransactionEvent, TransactionEventDelivery,
    TransactionEventKind, TransactionId, TransactionRecord,
};

impl EditorTransactionEngine {
    pub(crate) fn ensure_mutation_available(&self) -> Result<(), EditCommandError> {
        self.start_operation("preflight mutation")?;
        self.clear_operation();
        Ok(())
    }

    pub(crate) fn begin_exclusive_transition(
        &self,
        operation: &'static str,
    ) -> Result<ExclusiveTransition<'_>, EditCommandError> {
        self.flush_operation_group()?;
        self.start_operation(operation)?;
        let mut state = self.lock_state();
        if !state.active.is_empty() {
            self.clear_operation_locked(&mut state);
            return Err(EditCommandError::InvariantViolation {
                invariant: "exclusive editor transitions require no active transaction scope",
            });
        }
        drop(state);
        Ok(ExclusiveTransition {
            engine: self,
            not_send: PhantomData,
        })
    }

    pub(super) fn push(
        &self,
        scope: TransactionId,
        mut command: CommandBox,
    ) -> Result<(), EditCommandError> {
        self.start_operation("push command")?;
        let (mut context, mut active) = match self.take_top_scope(scope) {
            Ok(parts) => parts,
            Err(error) => {
                self.clear_operation();
                return Err(error);
            }
        };
        if let Err(apply_error) = command.apply(context.as_mut()) {
            let apply_effect = apply_error.effect;
            let apply_error = apply_error.source;
            if apply_effect == CommandEffect::Applied {
                if let Err(rollback_error) = command.revert(context.as_mut()) {
                    active.commands.push(command);
                    self.restore_active(context, active, true);
                    return Err(EditCommandError::RollbackFailed {
                        command_error: Box::new(apply_error),
                        rollback_error: Box::new(rollback_error.source),
                    });
                }
            }
            command.finalize(context.as_mut());
            return match Self::cancel_frame(&mut active, context.as_mut()) {
                Ok(()) => {
                    let root = active.root;
                    let event = Self::canceled_event(&active);
                    self.finish_canceled(context);
                    if root {
                        self.publish_event(event);
                    }
                    Err(apply_error)
                }
                Err(cancel_error) => {
                    self.restore_active(context, active, true);
                    Err(EditCommandError::RollbackFailed {
                        command_error: Box::new(apply_error),
                        rollback_error: Box::new(cancel_error),
                    })
                }
            };
        }

        let merge_mode = active.merge_mode;
        if Self::merge_command(&mut active.commands, merge_mode, command.as_ref()) {
            command.finalize(context.as_mut());
        } else {
            active.commands.push(command);
        }
        self.restore_active(context, active, false);
        Ok(())
    }

    pub(super) fn commit(&self, scope: TransactionId) -> Result<TransactionId, EditCommandError> {
        self.commit_after_apply(scope, |_| Ok(()))
    }

    pub(super) fn commit_after_apply(
        &self,
        scope: TransactionId,
        mut after_apply: impl FnMut(&SelectionSnapshot) -> Result<(), EditCommandError>,
    ) -> Result<TransactionId, EditCommandError> {
        self.start_operation("commit transaction")?;
        let history_mutation = {
            let mut state = self.lock_state();
            let Some(active) = state.active.last() else {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::ScopeClosed);
            };
            if active.id != scope {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::ScopeClosed);
            }
            if active.root && !active.commands.is_empty() {
                match Self::reserve_history_mutation(&state, active.history) {
                    Ok(reservation) => Some(reservation),
                    Err(error) => {
                        self.clear_operation_locked(&mut state);
                        return Err(error);
                    }
                }
            } else {
                None
            }
        };
        let (mut context, mut active) = match self.take_top_scope(scope) {
            Ok(parts) => parts,
            Err(error) => {
                self.clear_operation();
                return Err(error);
            }
        };
        let selection_after = context.selection_snapshot();
        if let Err(error) = after_apply(&selection_after) {
            let event = active.root.then(|| Self::canceled_event(&active));
            return match Self::cancel_frame(&mut active, context.as_mut()) {
                Ok(()) => {
                    self.finish_canceled(context);
                    if let Some(event) = event {
                        self.publish_event(event);
                    }
                    Err(error)
                }
                Err(rollback_error) => {
                    self.restore_active(context, active, true);
                    Err(EditCommandError::RollbackFailed {
                        command_error: Box::new(error),
                        rollback_error: Box::new(rollback_error),
                    })
                }
            };
        }
        let parent = self.lock_state().active.pop();
        if let Some(mut parent) = parent {
            let merge_mode = parent.merge_mode;
            for mut command in active.commands {
                if Self::merge_command(&mut parent.commands, merge_mode, command.as_ref()) {
                    command.finalize(context.as_mut());
                } else {
                    parent.commands.push(command);
                }
            }
            parent.participants.extend(active.participants);
            let id = active.id;
            self.restore_active(context, parent, false);
            return Ok(id);
        }

        let record = TransactionRecord {
            id: active.id,
            label: active.label.clone(),
            timestamp_frame: active.timestamp_frame,
            significant: active
                .commands
                .iter()
                .any(|command| command.is_significant()),
            commands: active.commands,
            participants: active.participants,
            selection_before: active.selection_before,
            selection_after,
        };
        let event = TransactionEvent {
            transaction: active.id,
            history: active.history,
            label: active.label.clone(),
            timestamp_frame: active.timestamp_frame,
            kind: TransactionEventKind::Committed,
        };
        let mut removed = Vec::new();
        {
            let mut state = self.lock_state();
            if !record.commands.is_empty() {
                let capacity = state.history_capacity;
                removed = state
                    .histories
                    .entry(active.history)
                    .or_insert_with(|| HistoryStore::from_validated_capacity(capacity))
                    .push(record);
                if let Some(reservation) = history_mutation {
                    Self::record_history_mutation(&mut state, active.history, reservation);
                }
            }
        }
        for record in &mut removed {
            record.finalize(context.as_mut());
        }
        let id = active.id;
        self.finish_operation(context, false);
        self.publish_event(event);
        Ok(id)
    }

    pub(super) fn cancel(&self, scope: TransactionId) -> Result<(), EditCommandError> {
        self.start_operation("cancel transaction")?;
        let (mut context, mut frames) = {
            let mut state = self.lock_state();
            let Some(position) = state.active.iter().position(|active| active.id == scope) else {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::ScopeClosed);
            };
            let context = Self::take_context_from(&mut state)?;
            let frames = state.active.drain(position..).collect::<Vec<_>>();
            (context, frames)
        };

        let mut event = None;
        while let Some(mut frame) = frames.pop() {
            match Self::cancel_frame(&mut frame, context.as_mut()) {
                Ok(()) => {
                    if frame.root {
                        event = Some(Self::canceled_event(&frame));
                    }
                }
                Err(error) => {
                    let mut state = self.lock_state();
                    for retained in frames {
                        state.active.push(retained);
                    }
                    state.active.push(frame);
                    state.context = Some(context);
                    state.faulted = true;
                    self.clear_operation_locked(&mut state);
                    return Err(error);
                }
            }
        }
        self.finish_operation(context, false);
        if let Some(event) = event {
            self.publish_event(event);
        }
        Ok(())
    }

    fn cancel_frame(
        frame: &mut ActiveTransaction,
        context: &mut dyn EditContext,
    ) -> Result<(), EditCommandError> {
        let original_selection = context.selection_snapshot();
        for index in (0..frame.commands.len()).rev() {
            if let Err(command_error) = frame.commands[index].revert(context) {
                let rollback_start = match command_error.effect {
                    CommandEffect::Unchanged => index + 1,
                    CommandEffect::Applied => index,
                };
                let command_error = command_error.source;
                for rollback_index in rollback_start..frame.commands.len() {
                    if let Err(rollback_error) = frame.commands[rollback_index].apply(context) {
                        return Err(EditCommandError::RollbackFailed {
                            command_error: Box::new(command_error),
                            rollback_error: Box::new(rollback_error.source),
                        });
                    }
                }
                return match context.restore_selection(&original_selection) {
                    Ok(()) => Err(command_error),
                    Err(rollback_error) => Err(EditCommandError::RollbackFailed {
                        command_error: Box::new(command_error),
                        rollback_error: Box::new(rollback_error),
                    }),
                };
            }
        }
        if let Err(command_error) = context.restore_selection(&frame.selection_before) {
            for command in &mut frame.commands {
                if let Err(rollback_error) = command.apply(context) {
                    return Err(EditCommandError::RollbackFailed {
                        command_error: Box::new(command_error),
                        rollback_error: Box::new(rollback_error.source),
                    });
                }
            }
            return match context.restore_selection(&original_selection) {
                Ok(()) => Err(command_error),
                Err(rollback_error) => Err(EditCommandError::RollbackFailed {
                    command_error: Box::new(command_error),
                    rollback_error: Box::new(rollback_error),
                }),
            };
        }
        for command in &mut frame.commands {
            command.finalize(context);
        }
        Ok(())
    }

    fn merge_command(commands: &mut [CommandBox], mode: MergeMode, next: &dyn EditCommand) -> bool {
        match mode {
            MergeMode::Disable => false,
            MergeMode::Ends => commands
                .last_mut()
                .is_some_and(|current| current.try_merge(next) == MergeOutcome::Merged),
            MergeMode::All => commands
                .iter_mut()
                .rev()
                .any(|current| current.try_merge(next) == MergeOutcome::Merged),
        }
    }

    fn take_top_scope(
        &self,
        scope: TransactionId,
    ) -> Result<(Box<dyn EditContext>, ActiveTransaction), EditCommandError> {
        let mut state = self.lock_state();
        if !state.active.last().is_some_and(|active| active.id == scope) {
            return Err(EditCommandError::ScopeClosed);
        }
        let active = state.active.pop().ok_or(EditCommandError::ScopeClosed)?;
        let context = Self::take_context_from(&mut state)?;
        Ok((context, active))
    }

    fn restore_active(
        &self,
        context: Box<dyn EditContext>,
        active: ActiveTransaction,
        faulted: bool,
    ) {
        let mut state = self.lock_state();
        state.active.push(active);
        state.context = Some(context);
        state.faulted |= faulted;
        self.clear_operation_locked(&mut state);
    }

    fn finish_canceled(&self, context: Box<dyn EditContext>) {
        let mut state = self.lock_state();
        state.context = Some(context);
        self.clear_operation_locked(&mut state);
    }

    fn canceled_event(active: &ActiveTransaction) -> TransactionEvent {
        TransactionEvent {
            transaction: active.id,
            history: active.history,
            label: active.label.clone(),
            timestamp_frame: active.timestamp_frame,
            kind: TransactionEventKind::Canceled,
        }
    }

    pub(super) fn publish_event(&self, event: TransactionEvent) {
        match self.event_sink.publish(event) {
            TransactionEventDelivery::Delivered => {}
            TransactionEventDelivery::Backpressured => {
                tracing::warn!("transaction lifecycle delivery is backpressured")
            }
            TransactionEventDelivery::Rejected => {
                tracing::warn!("transaction lifecycle delivery was rejected")
            }
        }
    }
}
