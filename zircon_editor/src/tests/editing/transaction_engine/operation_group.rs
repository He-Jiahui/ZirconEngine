use std::any::Any;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditCommandError, EditContext, EditorTransactionEngine,
    HistoryContextId, MergeMode, TransactionEvent, TransactionEventDelivery, TransactionEventKind,
    TransactionEventSink,
};
use crate::core::play::{PlayInstanceId, WorldDomain};

use super::fixture::{finalized_counter, DeltaCommand, FixtureContext};

#[derive(Default)]
struct RecordingEventSink {
    events: Mutex<Vec<TransactionEvent>>,
}

impl RecordingEventSink {
    fn kinds(&self) -> Vec<TransactionEventKind> {
        self.events
            .lock()
            .expect("the recording event sink lock should not be poisoned")
            .iter()
            .map(|event| event.kind)
            .collect()
    }
}

impl TransactionEventSink for RecordingEventSink {
    fn publish(&self, event: TransactionEvent) -> TransactionEventDelivery {
        self.events
            .lock()
            .expect("the recording event sink lock should not be poisoned")
            .push(event);
        TransactionEventDelivery::Delivered
    }
}

struct BlockingCommand {
    entered: FixedSignalSenderV1,
    release: FixedSignalReceiverV1,
}

impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for BlockingCommand {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        m.owned(&self.entered)?;
        m.owned(&self.release)?;
        Ok(())
    }
}
impl EditCommand for BlockingCommand {
    fn retention_plan_v1(
        &self,
        context: &dyn crate::core::editing::engine::EditContext,
        phase: crate::core::editing::engine::RetentionPhaseV1,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<
        crate::core::editing::engine::CommandRetentionPlanV1,
        crate::core::editing::engine::EditCommandError,
    > {
        crate::core::editing::engine::CommandRetentionPlanV1::for_owner(
            self, context, phase, model, limit,
        )
    }
    fn prepare_retention_v1(
        &mut self,
        _: &mut dyn crate::core::editing::engine::EditContext,
        phase: crate::core::editing::engine::RetentionPhaseV1,
        permit: &mut crate::core::editing::engine::RetentionMaterializationV1,
    ) -> Result<(), crate::core::editing::engine::CommandExecutionError> {
        permit
            .consume(self, phase)
            .map_err(crate::core::editing::engine::CommandExecutionError::unchanged)
    }

    fn label(&self) -> &str {
        "blocking operation group command"
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        self.entered.send(()).unwrap();
        self.release.recv_timeout(Duration::from_secs(5)).unwrap();
        let fixture = context
            .as_any_mut()
            .downcast_mut::<FixtureContext>()
            .ok_or(EditCommandError::ContextTypeMismatch {
                expected: "FixtureContext",
            })
            .map_err(CommandExecutionError::unchanged)?;
        fixture.value += 1;
        Ok(())
    }

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let fixture = context
            .as_any_mut()
            .downcast_mut::<FixtureContext>()
            .ok_or(EditCommandError::ContextTypeMismatch {
                expected: "FixtureContext",
            })
            .map_err(CommandExecutionError::unchanged)?;
        fixture.value -= 1;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[test]
fn operation_group_keeps_one_transaction_and_uses_command_merge() {
    let sink = Arc::new(RecordingEventSink::default());
    let engine = EditorTransactionEngine::with_event_sink(FixtureContext::default(), sink.clone());
    let finalized = finalized_counter();

    let first = engine
        .execute_operation(
            "Drag Value",
            HistoryContextId::Global,
            Some("fixture.drag.7"),
            MergeMode::All,
            Box::new(DeltaCommand::new("delta", 7, 2, finalized.clone())),
        )
        .unwrap();
    let second = engine
        .execute_operation(
            "Drag Value",
            HistoryContextId::Global,
            Some("fixture.drag.7"),
            MergeMode::All,
            Box::new(DeltaCommand::new("delta", 7, 3, finalized.clone())),
        )
        .unwrap();

    assert_eq!(first.transaction_id, second.transaction_id);
    assert!(first.group_open && second.group_open);
    assert_eq!(
        engine.flush_operation_group().unwrap(),
        Some(first.transaction_id)
    );
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(5)
    );
    let history = engine.history_status(HistoryContextId::Global).unwrap();
    assert_eq!(history.len, 1);
    assert_eq!(
        engine
            .history_details(HistoryContextId::Global, None, 1)
            .unwrap()
            .records()[0]
            .command_count,
        1
    );
    assert_eq!(finalized.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(
        sink.kinds(),
        vec![
            TransactionEventKind::Started,
            TransactionEventKind::Committed
        ]
    );
}

#[test]
fn operation_group_switch_commits_previous_and_undo_flushes_current() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let finalized = finalized_counter();

    let first = engine
        .execute_operation(
            "First Group",
            HistoryContextId::Global,
            Some("fixture.first"),
            MergeMode::Disable,
            Box::new(DeltaCommand::new("first", 1, 4, finalized.clone())),
        )
        .unwrap();
    let second = engine
        .execute_operation(
            "Second Group",
            HistoryContextId::Global,
            Some("fixture.second"),
            MergeMode::Disable,
            Box::new(DeltaCommand::new("second", 2, 6, finalized)),
        )
        .unwrap();

    assert_ne!(first.transaction_id, second.transaction_id);
    assert!(engine.undo(HistoryContextId::Global).unwrap());
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(4)
    );
    let history = engine.history_status(HistoryContextId::Global).unwrap();
    assert_eq!(history.len, 2);
    assert!(history.can_redo);
}

#[test]
fn operation_group_uses_the_play_history_route() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let instance = PlayInstanceId::for_test(29);
    let history = HistoryContextId::PlaySession(instance);

    let transaction = engine
        .execute_operation(
            "play grouped edit",
            history,
            Some("fixture.play-group"),
            MergeMode::Disable,
            Box::new(DeltaCommand::new(
                "play grouped edit",
                1,
                1,
                finalized_counter(),
            )),
        )
        .unwrap();

    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.world_domain)
            .unwrap(),
        Some(WorldDomain::Play(instance))
    );
    assert_eq!(
        engine.flush_operation_group().unwrap(),
        Some(transaction.transaction_id)
    );
}

#[test]
fn operation_group_flush_restores_group_after_generation_exhaustion() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let group = engine
        .execute_operation(
            "generation overflow",
            HistoryContextId::Global,
            Some("fixture.generation-overflow"),
            MergeMode::Disable,
            Box::new(DeltaCommand::new(
                "generation overflow",
                1,
                1,
                finalized_counter(),
            )),
        )
        .unwrap();
    engine.set_history_generation_for_test(HistoryContextId::Global, u64::MAX);

    assert!(matches!(
        engine.flush_operation_group(),
        Err(EditCommandError::HistoryGenerationExhausted {
            history: HistoryContextId::Global,
        })
    ));

    engine.set_history_generation_for_test(HistoryContextId::Global, 0);
    assert_eq!(
        engine.flush_operation_group().unwrap(),
        Some(group.transaction_id)
    );
}

#[test]
fn operation_group_flush_restores_group_after_concurrent_busy() {
    let engine = Arc::new(EditorTransactionEngine::new(FixtureContext::default()));
    let group = engine
        .execute_operation(
            "first grouped edit",
            HistoryContextId::Global,
            Some("fixture.concurrent-flush"),
            MergeMode::Disable,
            Box::new(DeltaCommand::new(
                "first grouped edit",
                1,
                1,
                finalized_counter(),
            )),
        )
        .unwrap();
    let (entered_tx, entered_rx) = fixed_signal_v1();
    let (release_tx, release_rx) = fixed_signal_v1();
    let worker_engine = Arc::clone(&engine);
    let worker = thread::spawn(move || {
        worker_engine.execute_operation(
            "continued grouped edit",
            HistoryContextId::Global,
            Some("fixture.concurrent-flush"),
            MergeMode::Disable,
            Box::new(BlockingCommand {
                entered: entered_tx,
                release: release_rx,
            }),
        )
    });
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();

    assert!(matches!(
        engine.flush_operation_group(),
        Err(EditCommandError::EngineBusy { .. })
    ));

    release_tx.send(()).unwrap();
    worker.join().unwrap().unwrap();
    assert_eq!(
        engine.flush_operation_group().unwrap(),
        Some(group.transaction_id)
    );
}

#[test]
fn operation_group_initialization_blocks_concurrent_flush() {
    let engine = Arc::new(EditorTransactionEngine::new(FixtureContext::default()));
    let (entered_tx, entered_rx) = fixed_signal_v1();
    let (release_tx, release_rx) = fixed_signal_v1();
    let worker_engine = Arc::clone(&engine);
    let worker = thread::spawn(move || {
        worker_engine.execute_operation(
            "initial grouped edit",
            HistoryContextId::Global,
            Some("fixture.initializing-flush"),
            MergeMode::Disable,
            Box::new(BlockingCommand {
                entered: entered_tx,
                release: release_rx,
            }),
        )
    });
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();

    assert!(matches!(
        engine.flush_operation_group(),
        Err(EditCommandError::EngineBusy {
            active: "initialize operation group",
            ..
        })
    ));

    release_tx.send(()).unwrap();
    let group = worker.join().unwrap().unwrap();
    assert_eq!(
        engine.flush_operation_group().unwrap(),
        Some(group.transaction_id)
    );
    assert_eq!(
        engine.history_status(HistoryContextId::Global).unwrap().len,
        1
    );
}

#[test]
fn operation_group_first_push_preserves_rollback_failure() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());

    let error = engine
        .execute_operation(
            "failing grouped edit",
            HistoryContextId::Global,
            Some("fixture.failing-first-push"),
            MergeMode::Disable,
            Box::new(
                DeltaCommand::new("failing grouped edit", 1, 5, finalized_counter())
                    .mutating_then_failing()
                    .revert_failing_before_mutation(),
            ),
        )
        .unwrap_err();

    match error {
        EditCommandError::RollbackFailed {
            command_error,
            rollback_error,
        } => {
            assert!(matches!(
                *command_error,
                EditCommandError::TargetMissing { ref target }
                    if target == "fixture after mutation"
            ));
            assert!(matches!(
                *rollback_error,
                EditCommandError::TargetMissing { ref target }
                    if target == "fixture revert before mutation"
            ));
        }
        other => panic!("expected rollback failure, got {other:?}"),
    }
    assert!(matches!(
        engine.history_status(HistoryContextId::Global),
        Err(EditCommandError::EngineFaulted { .. })
    ));
}

#[derive(Default)]
struct FixedSignalStateV1 {
    state: Mutex<(bool, bool)>,
    changed: std::sync::Condvar,
}
struct FixedSignalSenderV1(Arc<FixedSignalStateV1>);
struct FixedSignalReceiverV1(Arc<FixedSignalStateV1>);
fn fixed_signal_v1() -> (FixedSignalSenderV1, FixedSignalReceiverV1) {
    let state = Arc::new(FixedSignalStateV1::default());
    (
        FixedSignalSenderV1(state.clone()),
        FixedSignalReceiverV1(state),
    )
}
impl FixedSignalSenderV1 {
    fn send(&self, _: ()) -> Result<(), ()> {
        let mut state = self.0.state.lock().unwrap();
        if state.1 {
            return Err(());
        }
        state.0 = true;
        self.0.changed.notify_all();
        Ok(())
    }
}
impl FixedSignalReceiverV1 {
    fn recv_timeout(&self, timeout: Duration) -> Result<(), ()> {
        let state = self.0.state.lock().unwrap();
        let (state, _) = self
            .0
            .changed
            .wait_timeout_while(state, timeout, |s| !s.0 && !s.1)
            .unwrap();
        if state.0 {
            Ok(())
        } else {
            Err(())
        }
    }
}
impl Drop for FixedSignalSenderV1 {
    fn drop(&mut self) {
        self.0.state.lock().unwrap().1 = true;
        self.0.changed.notify_all();
    }
}
impl Drop for FixedSignalReceiverV1 {
    fn drop(&mut self) {
        self.0.state.lock().unwrap().1 = true;
        self.0.changed.notify_all();
    }
}
impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for FixedSignalStateV1 {
    fn retained_bound_v1(
        &self,
        _: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        Ok(())
    }
}
// The admitted Windows Mutex/Condvar representation and the two bools have no growable heap.
unsafe impl zircon_runtime_interface::retained_memory::ImmutableRetainedOwnerV1
    for FixedSignalStateV1
{
}
impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for FixedSignalSenderV1 {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        m.owned(&self.0)
    }
}
impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for FixedSignalReceiverV1 {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        m.owned(&self.0)
    }
}
