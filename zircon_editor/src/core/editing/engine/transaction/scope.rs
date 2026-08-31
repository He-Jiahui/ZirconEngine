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
        self.start_operation("begin transaction")?;
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
        if let Err(error) = context.activate_world_route(&route) {
            self.finish_operation(context, false);
            return Err(error);
        }
        let selection_before = context.selection_snapshot();
        let mut state = self.lock_state();
        let id = match Self::next_transaction_id(&mut state) {
            Ok(id) => id,
            Err(error) => {
                state.context = Some(context);
                self.clear_operation_locked(&mut state);
                return Err(error);
            }
        };
        let label = label.into();
        let root = state.active.is_empty();
        let timestamp_frame = state.current_frame;
        let mut participants = std::collections::BTreeSet::new();
        if let HistoryContextId::Document(document) = history {
            participants.insert(document);
        }
        state.active.push(ActiveTransaction {
            id,
            history,
            label: label.clone(),
            timestamp_frame,
            commands: Vec::new(),
            participants,
            selection_before,
            route,
            merge_mode: MergeMode::Disable,
            root,
        });
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

    pub(super) fn add_participant(&self, scope: TransactionId, document: DocumentId) {
        let mut state = self.lock_state();
        if state.operation.is_none() && !state.faulted {
            if let Some(active) = state.active.last_mut().filter(|active| active.id == scope) {
                active.participants.insert(document);
            }
        }
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

    pub fn add_participant(&mut self, document: DocumentId) {
        if !self.closed {
            self.engine.add_participant(self.id, document);
        }
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
    /// are published. An error cancels the active transaction through the normal rollback path.
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
                    self.engine.lock_state().drop_error = Some(error);
                    break;
                }
                Ok(()) => break,
            }
        }
        self.closed = true;
    }
}
