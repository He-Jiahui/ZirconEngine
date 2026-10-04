use std::any::Any;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use super::super::*;
use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditContext, EditWorldRoute, SelectionSnapshot,
};
use crate::core::play::{PlayInstanceId, WorldDomain};

#[derive(Default)]
struct Context;

impl EditContext for Context {
    fn capture_world_route(&self, domain: WorldDomain) -> Result<EditWorldRoute, EditCommandError> {
        Ok(EditWorldRoute::logical(domain))
    }
    fn activate_world_route(&mut self, _: &EditWorldRoute) -> Result<(), EditCommandError> {
        Ok(())
    }
    fn retire_world_route(&mut self, _: WorldDomain) -> Result<(), EditCommandError> {
        Ok(())
    }
    fn selection_snapshot(&self) -> SelectionSnapshot {
        SelectionSnapshot::default()
    }
    fn restore_selection(&mut self, _: &SelectionSnapshot) -> Result<(), EditCommandError> {
        Ok(())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct Increment(Arc<AtomicI64>);

impl EditCommand for Increment {
    fn label(&self) -> &str {
        "increment"
    }
    fn apply(&mut self, _: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn revert(&mut self, _: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        self.0.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn update(
    engine: &EditorTransactionEngine,
    history: HistoryContextId,
    value: &Arc<AtomicI64>,
) -> TransactionId {
    engine
        .execute_operation(
            "interaction",
            history,
            Some("interaction"),
            MergeMode::Disable,
            Box::new(Increment(Arc::clone(value))),
        )
        .unwrap()
        .transaction_id
}

#[test]
fn ordinary_observers_preserve_pending_group_and_committed_history() {
    let engine = EditorTransactionEngine::new(Context);
    let history = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let baseline = engine.dirty_states_since(None).unwrap();
    let generation = engine.history_generation_snapshot(history).unwrap();
    let first = update(&engine, history, &value);
    for _ in 0..3 {
        assert!(engine.is_dirty(history).unwrap());
        let status = engine.history_status(history).unwrap();
        assert!(status.dirty);
        assert!(status.can_undo);
        assert!(!status.can_redo);
        assert_eq!(status.len, 0);
        assert_eq!(status.top, None);
        assert_eq!(status.generation, generation);
        let details = engine.history_details(history, None, 16).unwrap();
        assert!(details.records().is_empty());
        assert!(details.status().dirty);
        assert_eq!(
            engine.history_generation_snapshot(history).unwrap(),
            generation
        );
        assert!(matches!(
            engine.journal_transaction(history, first),
            Err(TransactionJournalError::TransactionNotFound { .. })
        ));
        let pending = engine.dirty_states_since(Some(baseline.cursor())).unwrap();
        assert_eq!(pending.kind(), HistoryDirtyBatchKind::Delta);
        assert_eq!(pending.states().len(), 1);
        assert!(pending.states()[0].is_dirty());
        assert_eq!(pending.states()[0].history_generation(), generation);
    }
    assert_eq!(update(&engine, history, &value), first);
    assert_eq!(value.load(Ordering::SeqCst), 2);
    assert_eq!(engine.flush_operation_group().unwrap(), Some(first));
    let committed = engine.history_status(history).unwrap();
    assert_eq!(committed.len, 1);
    assert_eq!(committed.top, Some(first));
    assert_eq!(committed.generation, generation + 1);
    assert!(engine.undo(history).unwrap());
    assert_eq!(value.load(Ordering::SeqCst), 0);
}

#[test]
fn dirty_cursor_observes_pending_only_history_then_cancelled_clean_state() {
    let engine = EditorTransactionEngine::new(Context);
    let history = HistoryContextId::Document(DocumentId::new(7101));
    let other = HistoryContextId::Document(DocumentId::new(7102));
    let value = Arc::new(AtomicI64::new(0));
    let baseline = engine.dirty_states_since(None).unwrap();
    let id = update(&engine, history, &value);
    let pending = engine.dirty_states_since(Some(baseline.cursor())).unwrap();
    assert_eq!(pending.kind(), HistoryDirtyBatchKind::Delta);
    assert_eq!(pending.states().len(), 1);
    assert_eq!(pending.states()[0].history(), history);
    assert!(pending.states()[0].is_dirty());
    assert_eq!(pending.states()[0].history_generation(), 0);
    let reset = engine.dirty_states_since(None).unwrap();
    assert_eq!(reset.kind(), HistoryDirtyBatchKind::Reset);
    assert_eq!(reset.states().len(), 1);
    assert!(reset.states()[0].is_dirty());
    assert_eq!(
        engine
            .dirty_states_since(Some(pending.cursor()))
            .unwrap()
            .kind(),
        HistoryDirtyBatchKind::Unchanged
    );
    assert!(!engine.is_dirty(other).unwrap());
    engine.cancel(id).unwrap();
    assert_eq!(value.load(Ordering::SeqCst), 0);
    let cancelled = engine.dirty_states_since(Some(pending.cursor())).unwrap();
    assert_eq!(cancelled.kind(), HistoryDirtyBatchKind::Delta);
    assert_eq!(cancelled.states().len(), 1);
    assert_eq!(cancelled.states()[0].history(), history);
    assert!(!cancelled.states()[0].is_dirty());
    assert_eq!(cancelled.states()[0].history_generation(), 0);
    assert_eq!(engine.history_status(history).unwrap().len, 0);
    assert_eq!(
        engine
            .dirty_states_since(Some(cancelled.cursor()))
            .unwrap()
            .kind(),
        HistoryDirtyBatchKind::Unchanged
    );
}

#[test]
fn pending_volatile_history_stays_clean_without_being_committed_by_queries() {
    let engine = EditorTransactionEngine::new(Context);
    let value = Arc::new(AtomicI64::new(0));
    let history = HistoryContextId::PlaySession(PlayInstanceId::for_test(63));
    let baseline = engine.dirty_states_since(None).unwrap();
    let id = update(&engine, history, &value);
    assert!(!engine.is_dirty(history).unwrap());
    let status = engine.history_status(history).unwrap();
    assert!(!status.dirty);
    assert_eq!(status.len, 0);
    assert_eq!(
        engine
            .dirty_states_since(Some(baseline.cursor()))
            .unwrap()
            .kind(),
        HistoryDirtyBatchKind::Unchanged
    );
    assert_eq!(update(&engine, history, &value), id);
    engine.cancel(id).unwrap();
    assert_eq!(value.load(Ordering::SeqCst), 0);
}

#[test]
fn observers_preserve_busy_and_fault_errors_without_committing_group() {
    let engine = EditorTransactionEngine::new(Context);
    let value = Arc::new(AtomicI64::new(0));
    let history = HistoryContextId::Global;
    let id = update(&engine, history, &value);
    engine.lock_state().operation = Some("apply command");
    assert!(matches!(
        engine.history_status(history),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.is_dirty(history),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.history_details(history, None, 1),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.history_generation_snapshot(history),
        Err(EditCommandError::EngineBusy { .. })
    ));
    assert!(matches!(
        engine.dirty_states_since(None),
        Err(EditCommandError::EngineBusy { .. })
    ));
    engine.lock_state().operation = None;
    engine.lock_state().faulted = true;
    assert!(matches!(
        engine.history_status(history),
        Err(EditCommandError::EngineFaulted { .. })
    ));
    assert!(matches!(
        engine.is_dirty(history),
        Err(EditCommandError::EngineFaulted { .. })
    ));
    assert!(matches!(
        engine.history_details(history, None, 1),
        Err(EditCommandError::EngineFaulted { .. })
    ));
    assert!(matches!(
        engine.history_generation_snapshot(history),
        Err(EditCommandError::EngineFaulted { .. })
    ));
    assert!(matches!(
        engine.dirty_states_since(None),
        Err(EditCommandError::EngineFaulted { .. })
    ));
    engine.lock_state().faulted = false;
    assert_eq!(update(&engine, history, &value), id);
    engine.cancel(id).unwrap();
    assert_eq!(value.load(Ordering::SeqCst), 0);
}

#[test]
fn empty_active_scope_is_clean_and_applied_scope_can_be_observed_then_cancelled() {
    let engine = EditorTransactionEngine::new(Context);
    let history = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let baseline = engine.dirty_states_since(None).unwrap();
    let mut scope = engine.begin("explicit scope", history).unwrap();
    assert!(!engine.is_dirty(history).unwrap());
    assert_eq!(
        engine
            .dirty_states_since(Some(baseline.cursor()))
            .unwrap()
            .kind(),
        HistoryDirtyBatchKind::Unchanged
    );
    scope.push(Increment(Arc::clone(&value))).unwrap();
    assert!(engine.is_dirty(history).unwrap());
    let pending = engine.dirty_states_since(Some(baseline.cursor())).unwrap();
    assert!(pending.states()[0].is_dirty());
    assert_eq!(engine.history_status(history).unwrap().len, 0);
    scope.cancel().unwrap();
    let cancelled = engine.dirty_states_since(Some(pending.cursor())).unwrap();
    assert_eq!(cancelled.kind(), HistoryDirtyBatchKind::Delta);
    assert!(!cancelled.states()[0].is_dirty());
    assert_eq!(value.load(Ordering::SeqCst), 0);
}

#[test]
fn dirty_cursor_observes_commit_after_pending_without_duplication() {
    let engine = EditorTransactionEngine::new(Context);
    let history = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let id = update(&engine, history, &value);
    let pending = engine.dirty_states_since(None).unwrap();
    assert!(pending.states()[0].is_dirty());
    assert_eq!(pending.states()[0].history_generation(), 0);
    assert_eq!(engine.flush_operation_group().unwrap(), Some(id));
    let committed = engine.dirty_states_since(Some(pending.cursor())).unwrap();
    assert_eq!(committed.kind(), HistoryDirtyBatchKind::Delta);
    assert_eq!(committed.states().len(), 1);
    assert_eq!(committed.states()[0].history_generation(), 1);
    assert!(committed.states()[0].is_dirty());
    assert_eq!(engine.history_status(history).unwrap().len, 1);
    assert_eq!(
        engine
            .dirty_states_since(Some(committed.cursor()))
            .unwrap()
            .kind(),
        HistoryDirtyBatchKind::Unchanged
    );
}
