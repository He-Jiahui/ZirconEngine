use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Condvar, Mutex, MutexGuard};

use crate::core::editor_message::DocumentId;

use super::{
    CommandBox, CommandEffect, EditCommand, EditCommandError, EditContext, HistoryContextId,
    HistorySnapshot, HistoryStore, MergeOutcome, SelectionSnapshot, TransactionEvent,
    TransactionEventKind, TransactionId, TransactionRecord,
};

const DEFAULT_HISTORY_CAPACITY: usize = 128;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MergeMode {
    #[default]
    Disable,
    Ends,
    All,
}

struct ActiveTransaction {
    id: TransactionId,
    history: HistoryContextId,
    label: String,
    timestamp_frame: u64,
    commands: Vec<CommandBox>,
    participants: BTreeSet<DocumentId>,
    selection_before: SelectionSnapshot,
    merge_mode: MergeMode,
    root: bool,
}

struct EngineState {
    // The context is taken out while client code runs so no engine lock crosses a callback.
    context: Option<Box<dyn EditContext>>,
    histories: BTreeMap<HistoryContextId, HistoryStore>,
    active: Vec<ActiveTransaction>,
    events: Vec<TransactionEvent>,
    next_transaction: u64,
    current_frame: u64,
    history_capacity: usize,
    operation: Option<&'static str>,
    // A failed recovery freezes mutation while retaining the active/history recovery state.
    faulted: bool,
    drop_error: Option<EditCommandError>,
}

pub struct EditorTransactionEngine {
    state: Mutex<EngineState>,
    operation_changed: Condvar,
}

impl EditorTransactionEngine {
    pub fn new(context: impl EditContext + 'static) -> Self {
        Self::build(context, DEFAULT_HISTORY_CAPACITY)
    }

    pub fn with_capacity(
        context: impl EditContext + 'static,
        history_capacity: usize,
    ) -> Result<Self, EditCommandError> {
        if history_capacity == 0 {
            return Err(EditCommandError::InvalidHistoryCapacity);
        }
        Ok(Self::build(context, history_capacity))
    }

    fn build(context: impl EditContext + 'static, history_capacity: usize) -> Self {
        Self {
            state: Mutex::new(EngineState {
                context: Some(Box::new(context)),
                histories: BTreeMap::new(),
                active: Vec::new(),
                events: Vec::new(),
                next_transaction: 1,
                current_frame: 0,
                history_capacity,
                operation: None,
                faulted: false,
                drop_error: None,
            }),
            operation_changed: Condvar::new(),
        }
    }

    pub fn set_frame(&self, frame: u64) -> Result<(), EditCommandError> {
        self.start_operation("set frame")?;
        let mut state = self.lock_state();
        state.current_frame = frame;
        self.clear_operation_locked(&mut state);
        Ok(())
    }

    pub fn begin(
        &self,
        label: impl Into<String>,
        history: HistoryContextId,
    ) -> Result<TransactionScope<'_>, EditCommandError> {
        self.start_operation("begin transaction")?;
        let mut context = self.take_context()?;
        let selection_before = context.selection_snapshot();
        let mut state = self.lock_state();
        if let Some(active_history) = state.active.last().map(|active| active.history) {
            if active_history != history {
                state.context = Some(context);
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::CrossContextNested {
                    active: active_history,
                    requested: history,
                });
            }
        }
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
        let mut participants = BTreeSet::new();
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
            merge_mode: MergeMode::Disable,
            root,
        });
        if root {
            state.events.push(TransactionEvent {
                transaction: id,
                history,
                label,
                timestamp_frame,
                kind: TransactionEventKind::Started,
            });
        }
        state.context = Some(context);
        self.clear_operation_locked(&mut state);
        Ok(TransactionScope {
            engine: self,
            id,
            closed: false,
            not_send: PhantomData,
        })
    }

    pub fn undo(&self, history: HistoryContextId) -> Result<bool, EditCommandError> {
        self.replay(history, true)
    }

    pub fn redo(&self, history: HistoryContextId) -> Result<bool, EditCommandError> {
        self.replay(history, false)
    }

    pub fn mark_saved(&self, history: HistoryContextId) -> Result<(), EditCommandError> {
        self.start_operation("mark saved")?;
        let mut state = self.lock_state();
        let capacity = state.history_capacity;
        if !state.histories.contains_key(&history) {
            state
                .histories
                .insert(history, HistoryStore::from_validated_capacity(capacity));
        }
        if let Some(store) = state.histories.get_mut(&history) {
            store.mark_saved();
        }
        self.clear_operation_locked(&mut state);
        Ok(())
    }

    pub fn is_dirty(&self, history: HistoryContextId) -> Result<bool, EditCommandError> {
        self.start_operation("query dirty state")?;
        let mut state = self.lock_state();
        let dirty = state
            .histories
            .get(&history)
            .is_some_and(HistoryStore::is_dirty);
        self.clear_operation_locked(&mut state);
        Ok(dirty)
    }

    pub fn history_snapshot(
        &self,
        history: HistoryContextId,
    ) -> Result<HistorySnapshot, EditCommandError> {
        self.start_operation("snapshot history")?;
        let mut state = self.lock_state();
        let snapshot = match state.histories.get(&history) {
            Some(store) => store.snapshot(),
            None => HistorySnapshot::empty(),
        };
        self.clear_operation_locked(&mut state);
        Ok(snapshot)
    }

    pub fn drain_events(&self) -> Result<Vec<TransactionEvent>, EditCommandError> {
        self.start_operation("drain transaction events")?;
        let mut state = self.lock_state();
        let events = std::mem::take(&mut state.events);
        self.clear_operation_locked(&mut state);
        Ok(events)
    }

    pub fn take_drop_error(&self) -> Option<EditCommandError> {
        self.lock_state().drop_error.take()
    }

    pub fn with_context<T: 'static, R>(
        &self,
        inspect: impl FnOnce(&T) -> R,
    ) -> Result<Option<R>, EditCommandError> {
        self.start_operation("inspect edit context")?;
        {
            let mut state = self.lock_state();
            if !state.active.is_empty() {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::InvariantViolation {
                    invariant: "public context inspection requires no active transaction scope",
                });
            }
        }
        let context = self.take_context()?;
        let result = context.as_any().downcast_ref::<T>().map(inspect);
        self.finish_operation(context, false);
        Ok(result)
    }

    fn replay(&self, history: HistoryContextId, undo: bool) -> Result<bool, EditCommandError> {
        let operation = if undo { "undo" } else { "redo" };
        self.start_operation(operation)?;
        let (mut context, mut store, timestamp_frame) = {
            let mut state = self.lock_state();
            if !state.active.is_empty() {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::InvariantViolation {
                    invariant: "undo and redo require no active transaction scope",
                });
            }
            let context = Self::take_context_from(&mut state)?;
            let Some(store) = state.histories.remove(&history) else {
                state.context = Some(context);
                self.clear_operation_locked(&mut state);
                return Ok(false);
            };
            (context, store, state.current_frame)
        };
        let result = if undo {
            store.undo(context.as_mut())
        } else {
            store.redo(context.as_mut())
        };
        let faulted = matches!(&result, Err(EditCommandError::RollbackFailed { .. }));
        let event = result.as_ref().ok().and_then(|record| {
            record.as_ref().map(|record| TransactionEvent {
                transaction: record.id,
                history,
                label: record.label.clone(),
                timestamp_frame,
                kind: if undo {
                    TransactionEventKind::UndoApplied
                } else {
                    TransactionEventKind::RedoApplied
                },
            })
        });
        let mut state = self.lock_state();
        state.histories.insert(history, store);
        if let Some(event) = event {
            state.events.push(event);
        }
        state.context = Some(context);
        state.faulted |= faulted;
        self.clear_operation_locked(&mut state);
        result.map(|record| record.is_some())
    }

    fn push(&self, scope: TransactionId, mut command: CommandBox) -> Result<(), EditCommandError> {
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
                    self.finish_canceled(context, root.then_some(event));
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

    fn commit(&self, scope: TransactionId) -> Result<TransactionId, EditCommandError> {
        self.start_operation("commit transaction")?;
        let (mut context, active) = match self.take_top_scope(scope) {
            Ok(parts) => parts,
            Err(error) => {
                self.clear_operation();
                return Err(error);
            }
        };
        let selection_after = context.selection_snapshot();
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
            }
            state.events.push(TransactionEvent {
                transaction: active.id,
                history: active.history,
                label: active.label,
                timestamp_frame: active.timestamp_frame,
                kind: TransactionEventKind::Committed,
            });
        }
        for record in &mut removed {
            record.finalize(context.as_mut());
        }
        let id = active.id;
        self.finish_operation(context, false);
        Ok(id)
    }

    fn cancel(&self, scope: TransactionId) -> Result<(), EditCommandError> {
        self.start_operation("cancel transaction")?;
        let (mut context, mut frames) = {
            let mut state = self.lock_state();
            let Some(position) = state.active.iter().position(|active| active.id == scope) else {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::ScopeClosed);
            };
            let context = Self::take_context_from(&mut state)?;
            let frames = state.active.drain(position..).rev().collect::<Vec<_>>();
            (context, frames)
        };

        while !frames.is_empty() {
            let mut frame = frames.remove(0);
            match Self::cancel_frame(&mut frame, context.as_mut()) {
                Ok(()) => {
                    if frame.root {
                        self.lock_state().events.push(Self::canceled_event(&frame));
                    }
                }
                Err(error) => {
                    let mut state = self.lock_state();
                    for retained in frames.into_iter().rev() {
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

    fn set_merge_mode(&self, scope: TransactionId, mode: MergeMode) {
        let mut state = self.lock_state();
        if state.operation.is_none() && !state.faulted {
            if let Some(active) = state.active.last_mut().filter(|active| active.id == scope) {
                active.merge_mode = mode;
            }
        }
    }

    fn add_participant(&self, scope: TransactionId, document: DocumentId) {
        let mut state = self.lock_state();
        if state.operation.is_none() && !state.faulted {
            if let Some(active) = state.active.last_mut().filter(|active| active.id == scope) {
                active.participants.insert(document);
            }
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

    fn finish_canceled(&self, context: Box<dyn EditContext>, event: Option<TransactionEvent>) {
        let mut state = self.lock_state();
        if let Some(event) = event {
            state.events.push(event);
        }
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

    fn start_operation(&self, requested: &'static str) -> Result<(), EditCommandError> {
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

    fn clear_operation(&self) {
        let mut state = self.lock_state();
        self.clear_operation_locked(&mut state);
    }

    fn clear_operation_locked(&self, state: &mut EngineState) {
        state.operation = None;
        self.operation_changed.notify_all();
    }

    fn wait_for_operation(&self) {
        let mut state = self.lock_state();
        while state.operation.is_some() {
            state = match self.operation_changed.wait(state) {
                Ok(state) => state,
                Err(poison) => poison.into_inner(),
            };
        }
    }

    fn take_context(&self) -> Result<Box<dyn EditContext>, EditCommandError> {
        Self::take_context_from(&mut self.lock_state())
    }

    fn take_context_from(
        state: &mut EngineState,
    ) -> Result<Box<dyn EditContext>, EditCommandError> {
        state
            .context
            .take()
            .ok_or(EditCommandError::InvariantViolation {
                invariant: "an active engine operation owns the edit context",
            })
    }

    fn finish_operation(&self, context: Box<dyn EditContext>, faulted: bool) {
        let mut state = self.lock_state();
        state.context = Some(context);
        state.faulted |= faulted;
        self.clear_operation_locked(&mut state);
    }

    fn has_active_scope(&self, scope: TransactionId) -> bool {
        self.lock_state()
            .active
            .iter()
            .any(|active| active.id == scope)
    }

    fn next_transaction_id(state: &mut EngineState) -> Result<TransactionId, EditCommandError> {
        let sequence = state.next_transaction;
        state.next_transaction = state
            .next_transaction
            .checked_add(1)
            .ok_or(EditCommandError::TransactionIdExhausted)?;
        Ok(TransactionId::from_sequence(sequence))
    }

    fn lock_state(&self) -> MutexGuard<'_, EngineState> {
        match self.state.lock() {
            Ok(state) => state,
            Err(poison) => poison.into_inner(),
        }
    }
}

pub struct TransactionScope<'engine> {
    engine: &'engine EditorTransactionEngine,
    id: TransactionId,
    closed: bool,
    not_send: PhantomData<Rc<()>>,
}

impl TransactionScope<'_> {
    pub fn push(&mut self, command: impl EditCommand + 'static) -> Result<(), EditCommandError> {
        let result = self.engine.push(self.id, Box::new(command));
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
