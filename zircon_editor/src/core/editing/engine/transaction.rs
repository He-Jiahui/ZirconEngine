use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use crate::core::editor_message::DocumentId;

use super::{
    CommandBox, CommandEffect, DetachedTransactionEventSink, EditCommand, EditCommandError,
    EditContext, HistoryContextId, HistoryDetailPage, HistoryPageCursor, HistoryStatus,
    HistoryStore, MergeOutcome, SelectionSnapshot, TransactionEvent, TransactionEventDelivery,
    TransactionEventKind, TransactionEventSink, TransactionId, TransactionJournal,
    TransactionJournalError, TransactionRecord,
};

mod dirty_batch;
mod exclusive_transition;
mod operation_group;
mod save_token;

pub use dirty_batch::{
    HistoryDirtyBatch, HistoryDirtyBatchKind, HistoryDirtyCursor, HistoryDirtyState,
};
pub(crate) use exclusive_transition::ExclusiveTransition;
pub use operation_group::OperationTransactionResult;
use operation_group::{ActiveOperationGroup, OperationGroupReservation};

const DEFAULT_HISTORY_CAPACITY: usize = 128;
pub const MAX_HISTORY_DETAIL_PAGE_SIZE: usize = 128;

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
    history_generations: BTreeMap<HistoryContextId, u64>,
    history_dirty: dirty_batch::HistoryDirtyJournal,
    active: Vec<ActiveTransaction>,
    operation_group: Option<ActiveOperationGroup>,
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
    save_token_lineage: Arc<()>,
    event_sink: Arc<dyn TransactionEventSink>,
}

impl EditorTransactionEngine {
    pub fn new(context: impl EditContext + 'static) -> Self {
        Self::with_event_sink(context, Arc::new(DetachedTransactionEventSink))
    }

    pub fn with_capacity(
        context: impl EditContext + 'static,
        history_capacity: usize,
    ) -> Result<Self, EditCommandError> {
        if history_capacity == 0 {
            return Err(EditCommandError::InvalidHistoryCapacity);
        }
        Ok(Self::build(
            context,
            history_capacity,
            Arc::new(DetachedTransactionEventSink),
        ))
    }

    pub fn with_event_sink(
        context: impl EditContext + 'static,
        event_sink: Arc<dyn TransactionEventSink>,
    ) -> Self {
        Self::build(context, DEFAULT_HISTORY_CAPACITY, event_sink)
    }

    fn build(
        context: impl EditContext + 'static,
        history_capacity: usize,
        event_sink: Arc<dyn TransactionEventSink>,
    ) -> Self {
        Self {
            state: Mutex::new(EngineState {
                context: Some(Box::new(context)),
                histories: BTreeMap::new(),
                history_generations: BTreeMap::new(),
                history_dirty: dirty_batch::HistoryDirtyJournal::default(),
                active: Vec::new(),
                operation_group: None,
                next_transaction: 1,
                current_frame: 0,
                history_capacity,
                operation: None,
                faulted: false,
                drop_error: None,
            }),
            operation_changed: Condvar::new(),
            save_token_lineage: Arc::new(()),
            event_sink,
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
        self.flush_operation_group()?;
        let id = self.begin_transaction(label, history, None)?;
        Ok(TransactionScope {
            engine: self,
            id,
            closed: false,
            not_send: PhantomData,
        })
    }

    fn begin_transaction(
        &self,
        label: impl Into<String>,
        history: HistoryContextId,
        operation_group_reservation: Option<&OperationGroupReservation>,
    ) -> Result<TransactionId, EditCommandError> {
        self.start_operation("begin transaction")?;
        let mut context = self.take_context()?;
        let selection_before = context.selection_snapshot();
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
        let event = root.then(|| TransactionEvent {
            transaction: id,
            history,
            label,
            timestamp_frame,
            kind: TransactionEventKind::Started,
        });
        state.context = Some(context);
        self.clear_operation_locked(&mut state);
        drop(state);
        if let Some(event) = event {
            self.publish_event(event);
        }
        Ok(id)
    }

    pub fn undo(&self, history: HistoryContextId) -> Result<bool, EditCommandError> {
        self.flush_operation_group()?;
        self.replay(history, true)
    }

    pub fn redo(&self, history: HistoryContextId) -> Result<bool, EditCommandError> {
        self.flush_operation_group()?;
        self.replay(history, false)
    }

    pub fn is_dirty(&self, history: HistoryContextId) -> Result<bool, EditCommandError> {
        self.flush_operation_group()?;
        self.start_operation("query dirty state")?;
        let mut state = self.lock_state();
        let dirty = state
            .histories
            .get(&history)
            .is_some_and(HistoryStore::is_dirty);
        self.clear_operation_locked(&mut state);
        Ok(dirty)
    }

    pub fn history_status(
        &self,
        history: HistoryContextId,
    ) -> Result<HistoryStatus, EditCommandError> {
        self.flush_operation_group()?;
        self.start_operation("query history status")?;
        let mut state = self.lock_state();
        let generation = Self::history_generation(&state, history);
        let status = match state.histories.get(&history) {
            Some(store) => store.status(generation),
            None => HistoryStatus::empty(generation),
        };
        self.clear_operation_locked(&mut state);
        Ok(status)
    }

    pub fn history_details(
        &self,
        history: HistoryContextId,
        cursor: Option<&HistoryPageCursor>,
        page_size: usize,
    ) -> Result<HistoryDetailPage, EditCommandError> {
        if page_size == 0 || page_size > MAX_HISTORY_DETAIL_PAGE_SIZE {
            return Err(EditCommandError::HistoryPageSizeOutOfRange {
                requested: page_size,
                maximum: MAX_HISTORY_DETAIL_PAGE_SIZE,
            });
        }
        if cursor.is_some_and(|cursor| !cursor.belongs_to(&self.save_token_lineage)) {
            return Err(EditCommandError::HistoryPageCursorEngineMismatch);
        }
        if let Some(cursor) = cursor.filter(|cursor| cursor.history() != history) {
            return Err(EditCommandError::HistoryPageCursorHistoryMismatch {
                cursor_history: cursor.history(),
                requested_history: history,
            });
        }

        self.flush_operation_group()?;
        self.start_operation("query history details")?;
        let mut state = self.lock_state();
        let generation = Self::history_generation(&state, history);
        if let Some(cursor) = cursor.filter(|cursor| cursor.generation() != generation) {
            self.clear_operation_locked(&mut state);
            return Err(EditCommandError::HistoryPageCursorStale {
                history,
                cursor_generation: cursor.generation(),
                current_generation: generation,
            });
        }
        let offset = cursor.map_or(0, HistoryPageCursor::offset);
        let (status, records, has_more) = match state.histories.get(&history) {
            Some(store) => {
                let status = store.status(generation);
                let (records, has_more) = store.detail_window(offset, page_size);
                (status, records, has_more)
            }
            None => (HistoryStatus::empty(generation), Vec::new(), false),
        };
        let next_cursor = has_more.then(|| {
            HistoryPageCursor::new(
                Arc::clone(&self.save_token_lineage),
                history,
                generation,
                offset.saturating_add(records.len()),
            )
        });
        self.clear_operation_locked(&mut state);
        Ok(HistoryDetailPage::new(status, records, next_cursor))
    }

    pub fn journal_transaction(
        &self,
        history: HistoryContextId,
        transaction: TransactionId,
    ) -> Result<TransactionJournal, TransactionJournalError> {
        self.flush_operation_group()
            .map_err(TransactionJournalError::from)?;
        self.start_operation("serialize transaction journal")
            .map_err(TransactionJournalError::from)?;
        let mut state = self.lock_state();
        let journal = match state.histories.get(&history) {
            Some(store) => store.journal(history, transaction),
            None => Err(TransactionJournalError::TransactionNotFound {
                history,
                transaction,
            }),
        };
        self.clear_operation_locked(&mut state);
        journal
    }

    /// Returns the monotonic generation used to guard a save for one history context.
    pub fn history_generation_snapshot(
        &self,
        history: HistoryContextId,
    ) -> Result<u64, EditCommandError> {
        self.flush_operation_group()?;
        self.start_operation("snapshot history generation")?;
        let mut state = self.lock_state();
        let generation = Self::history_generation(&state, history);
        self.clear_operation_locked(&mut state);
        Ok(generation)
    }

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

    pub fn with_context_mut<T: 'static, R>(
        &self,
        inspect: impl FnOnce(&mut T) -> R,
    ) -> Result<Option<R>, EditCommandError> {
        self.start_operation("mutate edit context")?;
        {
            let mut state = self.lock_state();
            if !state.active.is_empty() {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::InvariantViolation {
                    invariant: "public context mutation requires no active transaction scope",
                });
            }
        }
        let mut context = self.take_context()?;
        let result = context.as_any_mut().downcast_mut::<T>().map(inspect);
        self.finish_operation(context, false);
        Ok(result)
    }

    fn replay(&self, history: HistoryContextId, undo: bool) -> Result<bool, EditCommandError> {
        let operation = if undo { "undo" } else { "redo" };
        self.start_operation(operation)?;
        let (mut context, mut store, timestamp_frame, mutation) = {
            let mut state = self.lock_state();
            if !state.active.is_empty() {
                self.clear_operation_locked(&mut state);
                return Err(EditCommandError::InvariantViolation {
                    invariant: "undo and redo require no active transaction scope",
                });
            }
            let can_replay = state.histories.get(&history).is_some_and(|store| {
                if undo {
                    store.can_undo()
                } else {
                    store.can_redo()
                }
            });
            if !can_replay {
                self.clear_operation_locked(&mut state);
                return Ok(false);
            }
            let mutation = match Self::reserve_history_mutation(&state, history) {
                Ok(reservation) => reservation,
                Err(error) => {
                    self.clear_operation_locked(&mut state);
                    return Err(error);
                }
            };
            let context = Self::take_context_from(&mut state)?;
            let Some(store) = state.histories.remove(&history) else {
                state.context = Some(context);
                self.clear_operation_locked(&mut state);
                return Ok(false);
            };
            (context, store, state.current_frame, mutation)
        };
        let result = if undo {
            store.undo(context.as_mut())
        } else {
            store.redo(context.as_mut())
        };
        let faulted = matches!(&result, Err(EditCommandError::RollbackFailed { .. }));
        let event = result.as_ref().ok().and_then(|event_metadata| {
            event_metadata
                .as_ref()
                .map(|(transaction, label)| TransactionEvent {
                    transaction: *transaction,
                    history,
                    label: label.clone(),
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
        if result.as_ref().ok().is_some_and(Option::is_some) {
            Self::record_history_mutation(&mut state, history, mutation);
        }
        state.context = Some(context);
        state.faulted |= faulted;
        self.clear_operation_locked(&mut state);
        drop(state);
        if let Some(event) = event {
            self.publish_event(event);
        }
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

    fn commit(&self, scope: TransactionId) -> Result<TransactionId, EditCommandError> {
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

    fn cancel(&self, scope: TransactionId) -> Result<(), EditCommandError> {
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

    fn publish_event(&self, event: TransactionEvent) {
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

#[cfg(test)]
mod performance_source_guards {
    #[test]
    fn nested_cancel_does_not_remove_from_the_front_of_a_vec() {
        let source = include_str!("transaction.rs");
        let front_remove = ["frames", ".remove(0)"].concat();
        assert!(!source.contains(&front_remove));
    }
}
