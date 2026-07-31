use std::any::Any;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak, mpsc};
use std::time::Duration;

use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditCommandError, EditContext, EditorTransactionEngine,
    HistoryContextId,
};

use super::fixture::FixtureContext;

struct ReentrantCommand {
    engine: Weak<EditorTransactionEngine>,
    observed_busy: Arc<AtomicBool>,
}

impl EditCommand for ReentrantCommand {
    fn label(&self) -> &str {
        "reentrant"
    }

    fn apply(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let engine = self
            .engine
            .upgrade()
            .ok_or(EditCommandError::InvariantViolation {
                invariant: "the reentrant test engine remains alive",
            })
            .map_err(CommandExecutionError::unchanged)?;
        self.observed_busy.store(
            matches!(
                engine.history_status(HistoryContextId::Global),
                Err(EditCommandError::EngineBusy { .. })
            ),
            Ordering::SeqCst,
        );
        Ok(())
    }

    fn revert(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[test]
fn command_callback_reentry_returns_busy_without_deadlock() {
    let engine = Arc::new(EditorTransactionEngine::new(FixtureContext::default()));
    let observed_busy = Arc::new(AtomicBool::new(false));
    let mut scope = engine.begin("reentrant", HistoryContextId::Global).unwrap();
    scope
        .push(ReentrantCommand {
            engine: Arc::downgrade(&engine),
            observed_busy: observed_busy.clone(),
        })
        .unwrap();
    scope.cancel().unwrap();
    assert!(observed_busy.load(Ordering::SeqCst));
}

#[test]
fn public_context_callback_and_concurrent_query_have_deterministic_lock_order() {
    let engine = Arc::new(EditorTransactionEngine::new(FixtureContext::default()));
    let worker_engine = engine.clone();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        worker_engine
            .with_context::<FixtureContext, _>(|_| {
                entered_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(1)).unwrap();
            })
            .unwrap();
    });

    entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(matches!(
        engine.history_status(HistoryContextId::Global),
        Err(EditCommandError::EngineBusy { .. })
    ));
    release_tx.send(()).unwrap();
    worker.join().unwrap();
    assert!(engine.history_status(HistoryContextId::Global).is_ok());
}

#[test]
fn active_scope_rejects_context_callback_that_would_cancel_it() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let scope = engine.begin("cancel", HistoryContextId::Global).unwrap();
    let executed = Arc::new(AtomicBool::new(false));
    let executed_in_callback = executed.clone();
    assert!(matches!(
        engine.with_context::<FixtureContext, _>(move |_| {
            executed_in_callback.store(true, Ordering::SeqCst);
            scope.cancel().unwrap();
        }),
        Err(EditCommandError::InvariantViolation { .. })
    ));
    assert!(!executed.load(Ordering::SeqCst));
    engine
        .begin("after rejected cancel", HistoryContextId::Global)
        .unwrap()
        .cancel()
        .unwrap();
}

#[test]
fn active_scope_rejects_context_callback_that_would_commit_it() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let scope = engine.begin("commit", HistoryContextId::Global).unwrap();
    let executed = Arc::new(AtomicBool::new(false));
    let executed_in_callback = executed.clone();
    assert!(matches!(
        engine.with_context::<FixtureContext, _>(move |_| {
            executed_in_callback.store(true, Ordering::SeqCst);
            scope.commit().unwrap();
        }),
        Err(EditCommandError::InvariantViolation { .. })
    ));
    assert!(!executed.load(Ordering::SeqCst));
    engine
        .begin("after rejected commit", HistoryContextId::Global)
        .unwrap()
        .cancel()
        .unwrap();
}

#[test]
fn active_scope_rejects_context_callback_that_would_drop_it() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let scope = engine.begin("drop", HistoryContextId::Global).unwrap();
    let executed = Arc::new(AtomicBool::new(false));
    let executed_in_callback = executed.clone();
    assert!(matches!(
        engine.with_context::<FixtureContext, _>(move |_| {
            executed_in_callback.store(true, Ordering::SeqCst);
            drop(scope);
        }),
        Err(EditCommandError::InvariantViolation { .. })
    ));
    assert!(!executed.load(Ordering::SeqCst));
    engine
        .begin("after rejected drop", HistoryContextId::Global)
        .unwrap()
        .cancel()
        .unwrap();
}

#[test]
fn exclusive_transition_blocks_interleaved_engine_operations() {
    let engine = Arc::new(EditorTransactionEngine::new(FixtureContext::default()));
    let mut transition = engine
        .begin_exclusive_transition("test editor world transition")
        .unwrap();
    let worker_engine = engine.clone();
    let worker = std::thread::spawn(move || {
        assert!(matches!(
            worker_engine.begin("interleaved", HistoryContextId::Global),
            Err(EditCommandError::EngineBusy { .. })
        ));
        assert!(matches!(
            worker_engine.history_status(HistoryContextId::Global),
            Err(EditCommandError::EngineBusy { .. })
        ));
    });

    transition
        .clear_history_and_context::<FixtureContext>(
            HistoryContextId::Global,
            "FixtureContext",
            |context| {
                context.value = 41;
                Ok(())
            },
        )
        .unwrap();
    worker.join().unwrap();
    drop(transition);

    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(41)
    );
    assert!(engine.history_status(HistoryContextId::Global).is_ok());
}
