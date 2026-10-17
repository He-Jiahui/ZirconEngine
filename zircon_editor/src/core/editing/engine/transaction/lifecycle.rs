use super::{
    ActiveTransaction, CommandBox, CommandEffect, EditCommand, EditCommandError, EditContext,
    EditorTransactionEngine, HistoryContextId, HistoryStore, MergeMode, MergeOutcome,
    SelectionSnapshot, TransactionEvent, TransactionEventDelivery, TransactionEventKind,
    TransactionId, TransactionRecord,
};

impl EditorTransactionEngine {
    pub(crate) fn ensure_mutation_available(&self) -> Result<(), EditCommandError> {
        self.start_operation("preflight mutation")?;
        self.clear_operation();
        Ok(())
    }

    pub(super) fn with_active_context_mut<T: 'static, R>(
        &self,
        scope: TransactionId,
        inspect: impl FnOnce(&mut T) -> R,
    ) -> Result<Option<R>, EditCommandError> {
        self.start_operation("inspect active transaction context")?;
        let (mut context, active) = match self.take_top_scope(scope) {
            Ok(parts) => parts,
            Err(error) => {
                self.clear_operation();
                return Err(error);
            }
        };
        let result = context.as_any_mut().downcast_mut::<T>().map(inspect);
        self.finish_active(context, active, false);
        Ok(result)
    }

    pub(super) fn push(
        &self,
        scope: TransactionId,
        mut command: CommandBox,
    ) -> Result<(), EditCommandError> {
        self.start_operation("push command")?;
        // Reserve before client code mutates the world; never wrap into a stale token.
        let applied_revision = {
            let mut state = self.lock_state();
            match state.applied_command_revision.checked_add(1) {
                Some(revision) => revision,
                None => {
                    self.clear_operation_locked(&mut state);
                    return Err(EditCommandError::AppliedCommandRevisionExhausted);
                }
            }
        };
        let (mut context, mut active) = match self.take_top_scope(scope) {
            Ok(parts) => parts,
            Err(error) => {
                self.clear_operation();
                return Err(error);
            }
        };
        let checkpoint = active.retention.checkpoint_v1();
        let prepared = (|| {
            let limits = self.incoming_retention_limits_v1(&active)?;
            let plan = command.retention_plan_with_limits_v1(
                context.as_ref(),
                super::super::RetentionPhaseV1::Apply,
                active.retention.model(),
                limits,
            )?;
            self.reserve_active_retention_v1(&active, Some(&plan), 0)?;
            let selection_bytes = plan.selection_bytes;
            // Allocation happens after admission; the supported Vec allocator returns the requested capacity.
            let mut permit =
                super::super::RetentionMaterializationV1::new(plan, active.retention.model());
            command
                .prepare_retention_v1(
                    context.as_mut(),
                    super::super::RetentionPhaseV1::Apply,
                    &mut permit,
                )
                .map_err(|error| error.into_source())?;
            permit.finish_v1()?;
            active.commands.try_reserve_exact(1).map_err(|_| {
                crate::core::editing::engine::retention::RetentionContractError::Allocation
            })?;
            Ok::<_, EditCommandError>(selection_bytes)
        })();
        let selection_bytes = match prepared {
            Ok(bytes) => bytes,
            Err(error) => {
                // Rejected ownership is dropped before withdrawing the reservation, outside both locks.
                drop(command);
                self.restore_active_reservation_v1(&active, checkpoint);
                self.finish_active(context, active, false);
                return Err(error);
            }
        };
        active.selection_ceiling_v1 = active
            .selection_ceiling_v1
            .checked_add(selection_bytes)
            .expect("the checked aggregate selection bound was admitted");
        if let Err(apply_error) = command.apply(context.as_mut()) {
            let apply_effect = apply_error.effect;
            let apply_error = apply_error.into_source();
            if apply_error.is_interrupted() {
                active.commands.push(command);
                self.finish_active(context, active, true);
                return Err(apply_error);
            }
            if apply_effect == CommandEffect::Applied {
                if let Err(rollback_error) =
                    crate::core::editing::engine::retention::prepare_under_owner_v1(
                        command.as_mut(),
                        context.as_mut(),
                        &active.retention,
                        super::super::RetentionPhaseV1::Rollback,
                    )
                    .and_then(|()| command.revert(context.as_mut()))
                {
                    active.commands.push(command);
                    self.finish_active(context, active, true);
                    return Err(EditCommandError::RollbackFailed {
                        command_error: Box::new(apply_error),
                        rollback_error: Box::new(rollback_error.into_source()),
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
                    self.finish_active(context, active, true);
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
            drop(command);
        } else {
            active.commands.push(command);
        }
        self.lock_state().applied_command_revision = applied_revision;
        self.finish_active(context, active, false);
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
        let mut event = None;
        let result = self.commit_after_apply_reserved(scope, &mut after_apply, &mut event);
        self.clear_operation();
        if let Some(event) = event {
            self.publish_event(event);
        }
        result
    }

    /// Executes commit under its caller's reservation without releasing admission.
    pub(super) fn commit_after_apply_reserved(
        &self,
        scope: TransactionId,
        after_apply: &mut impl FnMut(&SelectionSnapshot) -> Result<(), EditCommandError>,
        deferred_event: &mut Option<TransactionEvent>,
    ) -> Result<TransactionId, EditCommandError> {
        let history_mutation = {
            let state = self.lock_state();
            let Some(active) = state.active.last() else {
                return Err(EditCommandError::ScopeClosed);
            };
            if active.id != scope {
                return Err(EditCommandError::ScopeClosed);
            }
            if active.root && !active.commands.is_empty() {
                match Self::reserve_history_mutation(&state, active.history) {
                    Ok(reservation) => Some(reservation),
                    Err(error) => {
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
                return Err(error);
            }
        };
        let checkpoint = active.retention.checkpoint_v1();
        let previous_ceiling = active.selection_ceiling_v1;
        let selection_after = (|| {
            let bound = context.selection_retention_bound_v1(
                active.retention.model(),
                active.retention.budget().record_bytes,
            )?;
            active.selection_ceiling_v1 = previous_ceiling.max(bound);
            self.reserve_active_retention_v1(&active, None, 0)?;
            active.retention.capture_selection_v1(context.as_ref())
        })();
        let selection_after = match selection_after {
            Ok(selection) => selection,
            Err(error) => {
                active.selection_ceiling_v1 = previous_ceiling;
                self.restore_active_reservation_v1(&active, checkpoint);
                self.restore_active(context, active, false);
                return Err(error);
            }
        };
        if let Err(error) = after_apply(&selection_after) {
            if error.is_interrupted() {
                self.restore_active(context, active, true);
                return Err(error);
            }
            let event = active.root.then(|| Self::canceled_event(&active));
            return match Self::cancel_frame(&mut active, context.as_mut()) {
                Ok(()) => {
                    self.restore_operation_context(context, false);
                    if let Some(event) = event {
                        *deferred_event = Some(event);
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
            parent.commands.reserve_exact(active.commands.len());
            for mut command in active.commands {
                if Self::merge_command(&mut parent.commands, merge_mode, command.as_ref()) {
                    command.finalize(context.as_mut());
                } else {
                    parent.commands.push(command);
                }
            }
            parent.participants.extend(active.participants);
            parent.selection_ceiling_v1 = parent
                .selection_ceiling_v1
                .checked_add(active.selection_ceiling_v1)
                .expect("nested selection bound was admitted");
            let id = active.id;
            self.restore_active(context, parent, false);
            return Ok(id);
        }

        let record = TransactionRecord {
            id: active.id,
            label: active.label,
            timestamp_frame: active.timestamp_frame,
            route: active.route,
            significant: active
                .commands
                .iter()
                .any(|command| command.is_significant()),
            commands: active.commands,
            participants: active.participants,
            selection_before: active.selection_before,
            selection_after,
            retention: Some(active.retention),
        };
        let event = TransactionEvent {
            transaction: active.id,
            history: active.history,
            label: record.label.clone(),
            timestamp_frame: active.timestamp_frame,
            kind: TransactionEventKind::Committed,
        };
        let mut removed = Vec::new();
        {
            let mut state = self.lock_state();
            if !record.commands.is_empty() {
                removed = state
                    .histories
                    .get_mut(&active.history)
                    .expect("history storage was admitted before the scope")
                    .push_reserved_v1(record);
                if let Some(reservation) = history_mutation {
                    Self::record_history_mutation(&mut state, active.history, reservation);
                }
            }
        }
        for record in &mut removed {
            record.finalize(context.as_mut());
        }
        drop(removed); // payload Drop and last-owner release precede unlocking admission
        let id = active.id;
        self.restore_operation_context(context, false);
        *deferred_event = Some(event);
        Ok(id)
    }

    pub(super) fn cancel(&self, scope: TransactionId) -> Result<(), EditCommandError> {
        self.start_operation("cancel transaction")?;
        let (mut context, canceled_revision) = {
            let mut state = self.lock_state();
            if !state.active.iter().any(|active| active.id == scope) {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::ScopeClosed);
            }
            let changed = state
                .active
                .iter()
                .skip_while(|active| active.id != scope)
                .any(|active| !active.commands.is_empty());
            let canceled_revision = if changed {
                match state.applied_command_revision.checked_add(1) {
                    Some(revision) => Some(revision),
                    None => {
                        self.clear_operation_locked(&mut state);
                        return Err(EditCommandError::AppliedCommandRevisionExhausted);
                    }
                }
            } else {
                None
            };
            (Self::take_context_from(&mut state)?, canceled_revision)
        };

        let mut event = None;
        loop {
            let mut frame = {
                let mut state = self.lock_state();
                let Some(frame) = state.active.pop() else {
                    state.context = Some(context);
                    state.faulted = true;
                    self.clear_operation_locked(&mut state);
                    return Err(EditCommandError::InvariantViolation {
                        invariant: "nested cancellation must retain its target scope",
                    });
                };
                frame
            };
            let reached_scope = frame.id == scope;
            match Self::cancel_frame(&mut frame, context.as_mut()) {
                Ok(()) => {
                    if frame.root {
                        event = Some(Self::canceled_event(&frame));
                    }
                    if reached_scope {
                        break;
                    }
                }
                Err(error) => {
                    let mut state = self.lock_state();
                    state.active.push(frame);
                    state.context = Some(context);
                    state.faulted = true;
                    self.clear_operation_locked(&mut state);
                    return Err(error);
                }
            }
        }
        if let Some(revision) = canceled_revision {
            self.lock_state().applied_command_revision = revision;
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
        let original_selection = frame.retention.capture_selection_v1(context)?;
        for index in (0..frame.commands.len()).rev() {
            if let Err(command_error) =
                crate::core::editing::engine::retention::prepare_under_owner_v1(
                    frame.commands[index].as_mut(),
                    context,
                    &frame.retention,
                    super::super::RetentionPhaseV1::Rollback,
                )
                .and_then(|()| frame.commands[index].revert(context))
            {
                if command_error.effect == CommandEffect::Interrupted
                    || command_error.source.is_interrupted()
                {
                    return Err(command_error.into_source());
                }
                let rollback_start = match command_error.effect {
                    CommandEffect::Unchanged => index + 1,
                    CommandEffect::Applied => index,
                    CommandEffect::Interrupted => return Err(command_error.into_source()),
                };
                let command_error = command_error.into_source();
                for rollback_index in rollback_start..frame.commands.len() {
                    if let Err(rollback_error) =
                        crate::core::editing::engine::retention::prepare_under_owner_v1(
                            frame.commands[rollback_index].as_mut(),
                            context,
                            &frame.retention,
                            super::super::RetentionPhaseV1::Apply,
                        )
                        .and_then(|()| frame.commands[rollback_index].apply(context))
                    {
                        return Err(EditCommandError::RollbackFailed {
                            command_error: Box::new(command_error),
                            rollback_error: Box::new(rollback_error.into_source()),
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
            if command_error.is_interrupted() {
                return Err(command_error);
            }
            for command in &mut frame.commands {
                if let Err(rollback_error) =
                    crate::core::editing::engine::retention::prepare_under_owner_v1(
                        command.as_mut(),
                        context,
                        &frame.retention,
                        super::super::RetentionPhaseV1::Apply,
                    )
                    .and_then(|()| command.apply(context))
                {
                    return Err(EditCommandError::RollbackFailed {
                        command_error: Box::new(command_error),
                        rollback_error: Box::new(rollback_error.into_source()),
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

    pub(super) fn take_top_scope(
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
    }

    pub(super) fn finish_active(
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
