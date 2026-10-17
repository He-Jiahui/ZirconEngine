use std::marker::PhantomData;
use std::rc::Rc;

use super::{
    ActiveTransaction, CommandBox, DocumentId, EditCommand, EditCommandError, EditContext,
    EditorTransactionEngine, HistoryContextId, MergeMode, OperationGroupReservation,
    SelectionSnapshot, TransactionId,
};

pub struct TransactionScope<'engine> {
    engine: &'engine EditorTransactionEngine,
    id: TransactionId,
    closed: bool,
    not_send: PhantomData<Rc<()>>,
}

impl EditorTransactionEngine {
    pub fn begin(
        &self,
        label: impl Into<String>,
        history: HistoryContextId,
    ) -> Result<TransactionScope<'_>, EditCommandError> {
        self.flush_operation_group()?;
        let id = self.begin_transaction(label, history, None)?;
        Ok(TransactionScope {
            engine: self,
            id,
            closed: false,
            not_send: PhantomData,
        })
    }

    pub(super) fn begin_transaction(
        &self,
        label: impl Into<String>,
        history: HistoryContextId,
        operation_group_reservation: Option<&OperationGroupReservation>,
    ) -> Result<TransactionId, EditCommandError> {
        self.begin_transaction_with_retention_v1(label, history, operation_group_reservation, None)
    }

    pub(crate) fn begin_with_retention_v1(
        &self,
        label: impl Into<String>,
        history: HistoryContextId,
        retention: crate::core::editing::engine::retention::RootRetentionV1,
    ) -> Result<TransactionScope<'_>, EditCommandError> {
        self.flush_operation_group()?;
        let id = self.begin_transaction_with_retention_v1(label, history, None, Some(retention))?;
        Ok(TransactionScope {
            engine: self,
            id,
            closed: false,
            not_send: PhantomData,
        })
    }

    fn begin_transaction_with_retention_v1(
        &self,
        label: impl Into<String>,
        history: HistoryContextId,
        operation_group_reservation: Option<&OperationGroupReservation>,
        retention_override: Option<crate::core::editing::engine::retention::RootRetentionV1>,
    ) -> Result<TransactionId, EditCommandError> {
        self.start_operation("begin transaction")?;
        if let Err(error) = self
            .ensure_history_retention_v1(history)
            .and_then(|()| self.ensure_active_retention_slot_v1())
        {
            self.clear_operation();
            return Err(error);
        }
        let mut context = self.take_context()?;
        let inherited_route = {
            let mut state = self.lock_state();
            let operation_group_allows_begin = match state.operation_group.as_ref() {
                Some(active) => active.allows_begin(history, operation_group_reservation),
                None => operation_group_reservation.is_none(),
            };
            if !operation_group_allows_begin {
                let error = match state.operation_group.as_ref() {
                    Some(active) => EditCommandError::EngineBusy {
                        active: active.operation(),
                        requested: "begin transaction",
                    },
                    None => EditCommandError::InvariantViolation {
                        invariant: "operation group begin requires its live reservation",
                    },
                };
                state.context = Some(context);
                self.clear_operation_locked(&mut state);
                return Err(error);
            }
            if let Some(active) = state.active.last() {
                if active.history != history {
                    let active_history = active.history;
                    state.context = Some(context);
                    self.clear_operation_locked(&mut state);
                    return Err(EditCommandError::CrossContextNested {
                        active: active_history,
                        requested: history,
                    });
                }
            }
            state.active.last().map(|active| active.route.clone())
        };
        let route = match inherited_route {
            Some(route) => route,
            None => match context.capture_world_route(history.world_domain()) {
                Ok(route) => route,
                Err(error) => {
                    self.finish_operation(context, false);
                    return Err(error);
                }
            },
        };
        // Retained undo and redo records belong to one world lifetime. Reject a
        // replacement route before activating it or admitting any new commands.
        let route_check = {
            let state = self.lock_state();
            state.histories.get(&history).map_or(Ok(()), |store| {
                let stored = store.world_route()?;
                if stored.is_some_and(|stored| stored != &route) {
                    Err(EditCommandError::WorldRouteStale {
                        world_domain: history.world_domain(),
                    })
                } else {
                    Ok(())
                }
            })
        };
        if let Err(error) = route_check {
            self.finish_operation(context, false);
            return Err(error);
        }
        if let Err(error) = context.activate_world_route(&route) {
            self.finish_operation(context, false);
            return Err(error);
        }
        let label = label.into();
        let (id, root, timestamp_frame, inherited_retention) = {
            let mut state = self.lock_state();
            let id = match Self::next_transaction_id(&mut state) {
                Ok(id) => id,
                Err(error) => {
                    state.context = Some(context);
                    self.clear_operation_locked(&mut state);
                    return Err(error);
                }
            };
            (
                id,
                state.active.is_empty(),
                state.current_frame,
                state.active.last().map(|active| active.retention.clone()),
            )
        };
        let retention = match inherited_retention
            .or(retention_override)
            .map(Ok)
            .unwrap_or_else(|| {
                crate::core::editing::engine::retention::RootRetentionV1::new(
                    &self.retention_domain,
                    0,
                    0,
                )
            }) {
            Ok(owner) => owner,
            Err(error) => {
                self.finish_operation(context, false);
                return Err(error.into());
            }
        };
        if let Err(error) = retention.bind_history_v1(&self.retention_domain, history) {
            self.finish_operation(context, false);
            return Err(error.into());
        }
        let selection_bound = match context
            .selection_retention_bound_v1(retention.model(), retention.budget().record_bytes)
        {
            Ok(bound) => bound,
            Err(error) => {
                self.finish_operation(context, false);
                return Err(error);
            }
        };
        let checkpoint = retention.checkpoint_v1();
        if let Err(error) = retention.reserve_capture_v1(selection_bound) {
            self.finish_operation(context, false);
            return Err(error.into());
        }
        let selection_before = context.selection_snapshot();
        let mut selection_meter =
            crate::core::editing::engine::retention::RetainedBoundBuilderV1::with_limit(
                retention.model(),
                crate::core::editing::engine::retention::RetainedBytesV1::new(selection_bound),
            );
        if let Err(error) = selection_meter.owned(&selection_before) {
            drop(selection_before);
            self.restore_begin_reservation_v1(history, &retention, checkpoint);
            self.finish_operation(context, false);
            return Err(error.into());
        }
        let mut active = ActiveTransaction {
            id,
            history,
            label,
            timestamp_frame,
            commands: Vec::new(),
            participants: std::collections::BTreeSet::new(),
            selection_before,
            route,
            merge_mode: MergeMode::Disable,
            root,
            selection_ceiling_v1: selection_bound,
            retention,
        };
        if let Err(error) = self.reserve_active_retention_v1(&active, None, 0) {
            let retention = active.retention.clone();
            drop(active);
            self.restore_begin_reservation_v1(history, &retention, checkpoint);
            self.finish_operation(context, false);
            return Err(error);
        }
        if let HistoryContextId::Document(document) = history {
            active.participants.insert(document);
        }
        let label = active.label.clone();
        let mut state = self.lock_state();
        state.active.push(active);
        let event = root.then(|| super::TransactionEvent {
            transaction: id,
            history,
            label,
            timestamp_frame,
            kind: super::TransactionEventKind::Started,
        });
        state.context = Some(context);
        self.clear_operation_locked(&mut state);
        drop(state);
        if let Some(event) = event {
            self.publish_event(event);
        }
        Ok(id)
    }

    pub(super) fn set_merge_mode(&self, scope: TransactionId, mode: MergeMode) {
        let mut state = self.lock_state();
        if state.operation.is_none() && !state.faulted {
            if let Some(active) = state.active.last_mut().filter(|active| active.id == scope) {
                active.merge_mode = mode;
            }
        }
    }

    pub(super) fn add_participant(
        &self,
        scope: TransactionId,
        document: DocumentId,
    ) -> Result<(), EditCommandError> {
        self.start_operation("add transaction participant")?;
        let (context, mut active) = match self.take_top_scope(scope) {
            Ok(parts) => parts,
            Err(error) => {
                self.clear_operation();
                return Err(error);
            }
        };
        let result = (|| {
            if active.participants.contains(&document) {
                return Ok(());
            }
            let mut meter = crate::core::editing::engine::retention::RetainedBoundBuilderV1::new(
                active.retention.model(),
            );
            meter.btree_construction::<DocumentId, ()>(
                active.participants.len().checked_add(1).ok_or(
                    crate::core::editing::engine::retention::RetentionContractError::Overflow,
                )?,
            )?;
            self.reserve_active_retention_v1(&active, None, meter.bytes().get())?;
            active.participants.insert(document);
            Ok(())
        })();
        self.finish_active(context, active, false);
        result
    }
}

impl TransactionScope<'_> {
    /// Inspects the context only after this scope has pinned and activated its world route.
    pub(crate) fn with_context_mut<T: 'static, R>(
        &mut self,
        inspect: impl FnOnce(&mut T) -> R,
    ) -> Result<Option<R>, EditCommandError> {
        if self.closed {
            return Err(EditCommandError::ScopeClosed);
        }
        let result = self.engine.with_active_context_mut(self.id, inspect);
        if result.is_err() {
            self.closed = !self.engine.has_active_scope(self.id);
        }
        result
    }

    pub fn push(&mut self, command: impl EditCommand + 'static) -> Result<(), EditCommandError> {
        self.push_boxed(Box::new(command))
    }

    pub fn push_boxed(&mut self, command: CommandBox) -> Result<(), EditCommandError> {
        let result = self.engine.push(self.id, command);
        if result.is_err() {
            self.closed = !self.engine.has_active_scope(self.id);
        }
        result
    }

    pub fn set_merge_mode(&mut self, mode: MergeMode) {
        if !self.closed {
            self.engine.set_merge_mode(self.id, mode);
        }
    }

    pub fn add_participant(&mut self, document: DocumentId) -> Result<(), EditCommandError> {
        if self.closed {
            return Err(EditCommandError::ScopeClosed);
        }
        self.engine.add_participant(self.id, document)
    }

    pub fn cancel(mut self) -> Result<(), EditCommandError> {
        let result = loop {
            match self.engine.cancel(self.id) {
                Err(EditCommandError::EngineBusy { .. }) => self.engine.wait_for_operation(),
                result => break result,
            }
        };
        self.closed = true;
        result
    }

    pub fn commit(mut self) -> Result<TransactionId, EditCommandError> {
        let result = loop {
            match self.engine.commit(self.id) {
                Err(EditCommandError::EngineBusy { .. }) => self.engine.wait_for_operation(),
                result => break result,
            }
        };
        if result.is_ok() {
            self.closed = true;
        }
        result
    }

    /// Completes the transaction only after the applied selection has been accepted by the host.
    ///
    /// The callback runs after commands have applied but before history and lifecycle commit state
    /// are published. An ordinary error cancels the active transaction through normal rollback.
    /// A lifecycle interruption retains the active commands and faults the engine without
    /// compensation; effects already published by callbacks are not rolled back.
    pub fn commit_after_apply(
        mut self,
        mut after_apply: impl FnMut(&SelectionSnapshot) -> Result<(), EditCommandError>,
    ) -> Result<TransactionId, EditCommandError> {
        let result = loop {
            match self.engine.commit_after_apply(self.id, &mut after_apply) {
                Err(EditCommandError::EngineBusy { .. }) => self.engine.wait_for_operation(),
                result => break result,
            }
        };
        if result.is_ok() || !self.engine.has_active_scope(self.id) {
            self.closed = true;
        }
        result
    }
}

impl Drop for TransactionScope<'_> {
    fn drop(&mut self) {
        if self.closed || !self.engine.has_active_scope(self.id) {
            return;
        }
        loop {
            match self.engine.cancel(self.id) {
                Err(EditCommandError::EngineBusy { .. }) => self.engine.wait_for_operation(),
                Err(error) => {
                    let report = error.bounded_failure_report_v1();
                    drop(error); // arbitrary diagnostic Drop never runs under the state lock
                    let previous = self.engine.lock_state().drop_error.replace(report);
                    drop(previous);
                    break;
                }
                Ok(()) => break,
            }
        }
        self.closed = true;
    }
}
