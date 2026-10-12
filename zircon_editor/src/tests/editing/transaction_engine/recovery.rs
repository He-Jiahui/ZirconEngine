//! 以应用、逆转和选区恢复的注入失败核对事务回滚边界；失败时历史游标、上下文和命令释放必须保持可解释状态。
use std::sync::atomic::Ordering;

use crate::core::editing::engine::{EditCommandError, EditorTransactionEngine, HistoryContextId};

use super::fixture::{finalized_counter, DeltaCommand, FixtureContext};

#[test]
fn undo_selection_restore_failure_recovers_context_and_keeps_cursor() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext {
        fail_selection_restore: Some(0),
        ..FixtureContext::default()
    });
    let mut scope = engine.begin("selection", HistoryContextId::Global).unwrap();
    scope
        .push(DeltaCommand::new("selection", 1, 1, finalized).selecting(9))
        .unwrap();
    scope.commit().unwrap();

    assert!(matches!(
        engine.undo(HistoryContextId::Global),
        Err(EditCommandError::InvariantViolation { .. })
    ));
    let history = engine.history_status(HistoryContextId::Global).unwrap();
    assert_eq!(history.top.map(|transaction| transaction.raw()), Some(1));
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| (context.value, context.selection))
            .unwrap(),
        Some((1, 9))
    );
}

#[test]
fn redo_selection_restore_failure_recovers_context_and_keeps_cursor() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext {
        fail_selection_restore: Some(9),
        ..FixtureContext::default()
    });
    let mut scope = engine.begin("selection", HistoryContextId::Global).unwrap();
    scope
        .push(DeltaCommand::new("selection", 1, 1, finalized).selecting(9))
        .unwrap();
    scope.commit().unwrap();
    engine.undo(HistoryContextId::Global).unwrap();

    assert!(matches!(
        engine.redo(HistoryContextId::Global),
        Err(EditCommandError::InvariantViolation { .. })
    ));
    let history = engine.history_status(HistoryContextId::Global).unwrap();
    assert_eq!(history.top, None);
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| (context.value, context.selection))
            .unwrap(),
        Some((0, 0))
    );
}

#[test]
fn apply_error_after_mutation_reverts_the_failing_command() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("apply failure", HistoryContextId::Global)
        .unwrap();
    assert!(matches!(
        scope.push(DeltaCommand::new("mutating failure", 1, 5, finalized).mutating_then_failing()),
        Err(EditCommandError::TargetMissing { .. })
    ));
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(0)
    );
    let replacement = engine
        .begin("replacement", HistoryContextId::Global)
        .unwrap();
    replacement.cancel().unwrap();
}

#[test]
fn apply_error_before_mutation_does_not_run_reverse_effect() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("unchanged apply failure", HistoryContextId::Global)
        .unwrap();
    assert!(matches!(
        scope.push(DeltaCommand::new("unchanged", 1, 5, finalized).failing()),
        Err(EditCommandError::TargetMissing { .. })
    ));
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(0)
    );
}

#[test]
fn revert_error_before_mutation_retains_applied_state_and_faults_engine() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("unchanged revert failure", HistoryContextId::Global)
        .unwrap();
    scope
        .push(
            DeltaCommand::new("unchanged revert failure", 1, 5, finalized.clone())
                .revert_failing_before_mutation(),
        )
        .unwrap();
    assert!(matches!(
        scope.cancel(),
        Err(EditCommandError::TargetMissing { .. })
    ));
    assert_eq!(finalized.load(Ordering::SeqCst), 0);
    assert!(matches!(
        engine.history_status(HistoryContextId::Global),
        Err(EditCommandError::EngineFaulted { .. })
    ));
}

#[test]
fn revert_error_retains_commands_and_faults_engine_without_finalizing() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine
        .begin("revert failure", HistoryContextId::Global)
        .unwrap();
    scope
        .push(
            DeltaCommand::new("revert failure", 1, 5, finalized.clone())
                .revert_failing_after_mutation(),
        )
        .unwrap();
    assert!(matches!(
        scope.cancel(),
        Err(EditCommandError::TargetMissing { .. })
    ));
    assert_eq!(finalized.load(Ordering::SeqCst), 0);
    assert!(matches!(
        engine.history_status(HistoryContextId::Global),
        Err(EditCommandError::EngineFaulted { .. })
    ));
}

mod lifecycle_interruption {
    use std::any::Any;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    use crate::core::editing::engine::{
        CommandEffect, CommandExecutionError, EditCommand, EditCommandError, EditContext,
        EditWorldRoute, EditorTransactionEngine, HistoryContextId, SelectionSnapshot,
    };
    use crate::core::play::WorldDomain;

    #[derive(Default)]
    struct Probe {
        calls: Mutex<Vec<String>>,
        selection_restores: AtomicUsize,
        finalized: AtomicUsize,
        dropped: AtomicUsize,
    }

    impl Probe {
        fn record(&self, label: &str, operation: &str) {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{label}.{operation}"));
        }

        fn calls(&self) -> Vec<String> {
            self.calls.lock().unwrap().clone()
        }

        fn clear_calls(&self) {
            self.calls.lock().unwrap().clear();
            self.selection_restores.store(0, Ordering::SeqCst);
        }

        fn assert_retained(&self) {
            assert_eq!(self.finalized.load(Ordering::SeqCst), 0);
            assert_eq!(self.dropped.load(Ordering::SeqCst), 0);
        }
    }

    struct Context {
        probe: Arc<Probe>,
        value: i32,
        selection: u64,
        restore_attempts: usize,
        interrupt_restore_at: Option<usize>,
    }

    impl Context {
        fn new(probe: Arc<Probe>) -> Self {
            Self {
                probe,
                value: 0,
                selection: 0,
                restore_attempts: 0,
                interrupt_restore_at: None,
            }
        }
    }

    impl EditContext for Context {
        fn capture_world_route(
            &self,
            world_domain: WorldDomain,
        ) -> Result<EditWorldRoute, EditCommandError> {
            Ok(EditWorldRoute::logical(world_domain))
        }

        fn activate_world_route(
            &mut self,
            _route: &EditWorldRoute,
        ) -> Result<(), EditCommandError> {
            Ok(())
        }

        fn retire_world_route(
            &mut self,
            _world_domain: WorldDomain,
        ) -> Result<(), EditCommandError> {
            Ok(())
        }

        fn selection_snapshot(&self) -> SelectionSnapshot {
            SelectionSnapshot::fixture_value(self.selection, self.selection)
        }

        fn restore_selection(
            &mut self,
            snapshot: &SelectionSnapshot,
        ) -> Result<(), EditCommandError> {
            self.restore_attempts += 1;
            self.probe.selection_restores.fetch_add(1, Ordering::SeqCst);
            if self.interrupt_restore_at == Some(self.restore_attempts) {
                return Err(
                    CommandExecutionError::interrupted(target("selection lifecycle")).into_source(),
                );
            }
            self.selection = snapshot.fixture_value_ref()?;
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    struct OwnedPayload(Arc<Probe>);

    impl Drop for OwnedPayload {
        fn drop(&mut self) {
            self.0.dropped.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[derive(Clone, Copy)]
    enum Failure {
        InterruptedLiteral,
        NestedMarker,
        Unchanged,
        Applied,
        GatewayApplied,
    }

    impl Failure {
        fn error(self) -> CommandExecutionError {
            match self {
                Self::InterruptedLiteral => CommandExecutionError {
                    effect: CommandEffect::Interrupted,
                    source: target("lifecycle fixture"),
                },
                Self::NestedMarker => {
                    CommandExecutionError::applied(EditCommandError::RollbackFailed {
                        command_error: Box::new(
                            CommandExecutionError::interrupted(target("nested lifecycle"))
                                .into_source(),
                        ),
                        rollback_error: Box::new(target("ordinary nested recovery")),
                    })
                }
                Self::Unchanged => CommandExecutionError::unchanged(target("ordinary fixture")),
                Self::Applied => CommandExecutionError::applied(target("ordinary fixture")),
                Self::GatewayApplied => {
                    CommandExecutionError::applied(EditCommandError::ExternalEffect {
                        source: Box::new(std::io::Error::other(
                            "normal post-callback gateway failure",
                        )),
                    })
                }
            }
        }
    }

    struct Command {
        label: &'static str,
        probe: Arc<Probe>,
        _payload: OwnedPayload,
        applies: usize,
        reverts: usize,
        fail_apply: Option<(usize, Failure)>,
        fail_revert: Option<(usize, Failure)>,
    }

    impl Command {
        fn new(label: &'static str, probe: &Arc<Probe>) -> Self {
            Self {
                label,
                probe: Arc::clone(probe),
                _payload: OwnedPayload(Arc::clone(probe)),
                applies: 0,
                reverts: 0,
                fail_apply: None,
                fail_revert: None,
            }
        }

        fn fail_apply(mut self, attempt: usize, failure: Failure) -> Self {
            self.fail_apply = Some((attempt, failure));
            self
        }

        fn fail_revert(mut self, attempt: usize, failure: Failure) -> Self {
            self.fail_revert = Some((attempt, failure));
            self
        }
    }

    impl EditCommand for Command {
        fn label(&self) -> &str {
            self.label
        }

        fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
            self.applies += 1;
            self.probe.record(self.label, "apply");
            let failure = self
                .fail_apply
                .filter(|(attempt, _)| *attempt == self.applies)
                .map(|(_, failure)| failure);
            let context = context.as_any_mut().downcast_mut::<Context>().unwrap();
            if !matches!(failure, Some(Failure::Unchanged)) {
                context.value += 1;
            }
            failure.map_or(Ok(()), |failure| Err(failure.error()))
        }

        fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
            self.reverts += 1;
            self.probe.record(self.label, "revert");
            let failure = self
                .fail_revert
                .filter(|(attempt, _)| *attempt == self.reverts)
                .map(|(_, failure)| failure);
            let context = context.as_any_mut().downcast_mut::<Context>().unwrap();
            if !matches!(failure, Some(Failure::Unchanged)) {
                context.value -= 1;
            }
            failure.map_or(Ok(()), |failure| Err(failure.error()))
        }

        fn finalize(&mut self, _context: &mut dyn EditContext) {
            self.probe.finalized.fetch_add(1, Ordering::SeqCst);
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    fn target(message: &str) -> EditCommandError {
        EditCommandError::TargetMissing {
            target: message.to_string(),
        }
    }

    fn expected(labels: &[&str], operation: &str) -> Vec<String> {
        labels
            .iter()
            .map(|label| format!("{label}.{operation}"))
            .collect()
    }

    fn assert_faulted(engine: &EditorTransactionEngine) {
        assert!(matches!(
            engine.begin("blocked", HistoryContextId::Global),
            Err(EditCommandError::EngineFaulted { .. })
        ));
        assert!(matches!(
            engine.undo(HistoryContextId::Global),
            Err(EditCommandError::EngineFaulted { .. })
        ));
        assert!(matches!(
            engine.redo(HistoryContextId::Global),
            Err(EditCommandError::EngineFaulted { .. })
        ));
    }

    fn assert_secondary(error: EditCommandError) {
        assert!(error.is_interrupted());
        let EditCommandError::RollbackFailed {
            command_error,
            rollback_error,
        } = error
        else {
            panic!("keep both primary and secondary recovery errors");
        };
        assert!(matches!(
            *command_error,
            EditCommandError::TargetMissing { .. }
        ));
        assert!(matches!(
            *rollback_error,
            EditCommandError::LifecycleInterrupted { .. }
        ));
    }

    #[test]
    fn interrupted_literal_is_normalized_without_double_wrapping() {
        let first = CommandExecutionError {
            effect: CommandEffect::Interrupted,
            source: target("literal"),
        }
        .into_source();
        let normalized = CommandExecutionError {
            effect: CommandEffect::Interrupted,
            source: first,
        }
        .into_source();
        let EditCommandError::LifecycleInterrupted {
            source,
            post_callback_errors,
        } = normalized
        else {
            panic!("public literal must retain a persistent cancellation");
        };
        assert!(matches!(*source, EditCommandError::TargetMissing { .. }));
        assert!(post_callback_errors.is_empty());
    }

    #[test]
    fn interrupted_factory_preserves_two_post_callback_errors_in_order() {
        let first = CommandExecutionError::interrupted_with_post_callback_error(
            target("producer lifecycle"),
            Some(target("gateway first")),
        );
        let second = CommandExecutionError::interrupted_with_post_callback_error(
            first.into_source(),
            Some(target("gateway second")),
        );
        assert_eq!(second.effect, CommandEffect::Interrupted);
        let EditCommandError::LifecycleInterrupted {
            source,
            post_callback_errors,
        } = second.into_source()
        else {
            panic!("keep the original producer cancellation");
        };
        assert!(
            matches!(*source, EditCommandError::TargetMissing { ref target }
            if target == "producer lifecycle")
        );
        let messages = post_callback_errors
            .into_iter()
            .map(|error| match error {
                EditCommandError::TargetMissing { target } => target,
                _ => panic!("post diagnostics must not invent rollback failures"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            messages,
            vec![
                String::from("gateway first"),
                String::from("gateway second")
            ]
        );
    }

    #[test]
    fn interrupted_factory_preserves_nested_rollback_with_boundary_diagnostics() {
        let original = EditCommandError::RollbackFailed {
            command_error: Box::new(target("original command")),
            rollback_error: Box::new(
                CommandExecutionError::interrupted(target("original rollback")).into_source(),
            ),
        };
        let preserved = CommandExecutionError::interrupted(original).into_source();
        assert!(matches!(preserved, EditCommandError::RollbackFailed { .. }));
        let boundary = CommandExecutionError::interrupted_with_post_callback_error(
            preserved,
            Some(target("boundary gateway")),
        )
        .into_source();
        let EditCommandError::LifecycleInterrupted {
            source,
            post_callback_errors,
        } = boundary
        else {
            panic!("one new boundary must retain its post-callback diagnostic");
        };
        assert!(matches!(*source, EditCommandError::RollbackFailed { .. }));
        assert_eq!(post_callback_errors.len(), 1);
    }

    #[test]
    fn is_interrupted_checks_both_nested_rollback_sources() {
        for marker_is_primary in [false, true] {
            let marker = CommandExecutionError::interrupted(target("lifecycle")).into_source();
            let ordinary = target("ordinary");
            let (primary, secondary) = if marker_is_primary {
                (marker, ordinary)
            } else {
                (ordinary, marker)
            };
            let inner = EditCommandError::RollbackFailed {
                command_error: Box::new(primary),
                rollback_error: Box::new(secondary),
            };
            let outer = EditCommandError::RollbackFailed {
                command_error: Box::new(inner),
                rollback_error: Box::new(target("outer ordinary")),
            };
            assert!(outer.is_interrupted());
        }
        assert!(!target("ordinary").is_interrupted());
    }

    #[test]
    fn interrupted_push_retains_move_only_payload_and_scope_drop_stops() {
        let probe = Arc::new(Probe::default());
        let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
        let mut scope = engine
            .begin("push interrupted", HistoryContextId::Global)
            .unwrap();
        scope.push(Command::new("prior", &probe)).unwrap();
        let error = scope
            .push(Command::new("interrupted", &probe).fail_apply(1, Failure::InterruptedLiteral))
            .unwrap_err();
        assert!(error.is_interrupted());
        let calls = expected(&["prior", "interrupted"], "apply");
        assert_eq!(probe.calls(), calls);
        probe.assert_retained();
        assert_faulted(&engine);
        drop(scope);
        assert_eq!(probe.calls(), calls);
        assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 0);
        probe.assert_retained();
        drop(engine);
        assert_eq!(probe.dropped.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn interrupted_push_rejects_commit_and_save_token_work() {
        let probe = Arc::new(Probe::default());
        let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
        let old_save_token = engine.capture_save_token(HistoryContextId::Global).unwrap();
        let mut scope = engine
            .begin("fault gate", HistoryContextId::Global)
            .unwrap();
        assert!(scope
            .push(Command::new("interrupted", &probe).fail_apply(1, Failure::InterruptedLiteral))
            .unwrap_err()
            .is_interrupted());
        assert!(matches!(
            engine.capture_save_token(HistoryContextId::Global),
            Err(EditCommandError::EngineFaulted { .. })
        ));
        assert!(matches!(
            engine.mark_saved_if_unchanged(HistoryContextId::Global, old_save_token),
            Err(EditCommandError::EngineFaulted { .. })
        ));
        assert!(matches!(
            scope.commit(),
            Err(EditCommandError::EngineFaulted { .. })
        ));
        assert_eq!(probe.calls(), expected(&["interrupted"], "apply"));
        probe.assert_retained();
        drop(engine);
        assert_eq!(probe.dropped.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn interrupted_undo_stops_at_every_command_position_and_retains_payloads() {
        let labels = ["first", "middle", "last"];
        for canceled in 0..labels.len() {
            let probe = Arc::new(Probe::default());
            let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
            let mut scope = engine
                .begin("undo interrupted", HistoryContextId::Global)
                .unwrap();
            for (index, label) in labels.iter().copied().enumerate() {
                let mut command = Command::new(label, &probe);
                if index == canceled {
                    command = command.fail_revert(1, Failure::InterruptedLiteral);
                }
                scope.push(command).unwrap();
            }
            scope.commit().unwrap();
            probe.clear_calls();
            assert!(engine
                .undo(HistoryContextId::Global)
                .unwrap_err()
                .is_interrupted());
            let reversed = labels[canceled..].iter().copied().rev().collect::<Vec<_>>();
            assert_eq!(probe.calls(), expected(&reversed, "revert"));
            assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 0);
            probe.assert_retained();
            assert_faulted(&engine);
            assert_eq!(probe.calls(), expected(&reversed, "revert"));
            drop(engine);
            assert_eq!(probe.dropped.load(Ordering::SeqCst), 3);
        }
    }

    #[test]
    fn interrupted_redo_stops_at_every_command_position_and_retains_payloads() {
        let labels = ["first", "middle", "last"];
        for canceled in 0..labels.len() {
            let probe = Arc::new(Probe::default());
            let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
            let mut scope = engine
                .begin("redo interrupted", HistoryContextId::Global)
                .unwrap();
            for (index, label) in labels.iter().copied().enumerate() {
                let mut command = Command::new(label, &probe);
                if index == canceled {
                    command = command.fail_apply(2, Failure::InterruptedLiteral);
                }
                scope.push(command).unwrap();
            }
            scope.commit().unwrap();
            engine.undo(HistoryContextId::Global).unwrap();
            probe.clear_calls();
            assert!(engine
                .redo(HistoryContextId::Global)
                .unwrap_err()
                .is_interrupted());
            assert_eq!(probe.calls(), expected(&labels[..=canceled], "apply"));
            assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 0);
            probe.assert_retained();
            assert_faulted(&engine);
            assert_eq!(probe.calls(), expected(&labels[..=canceled], "apply"));
            drop(engine);
            assert_eq!(probe.dropped.load(Ordering::SeqCst), 3);
        }
    }

    #[test]
    fn interrupted_cancel_stops_at_every_command_position_and_retains_payloads() {
        let labels = ["first", "middle", "last"];
        for canceled in 0..labels.len() {
            let probe = Arc::new(Probe::default());
            let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
            let mut scope = engine
                .begin("cancel interrupted", HistoryContextId::Global)
                .unwrap();
            for (index, label) in labels.iter().copied().enumerate() {
                let mut command = Command::new(label, &probe);
                if index == canceled {
                    command = command.fail_revert(1, Failure::InterruptedLiteral);
                }
                scope.push(command).unwrap();
            }
            probe.clear_calls();
            assert!(scope.cancel().unwrap_err().is_interrupted());
            let reversed = labels[canceled..].iter().copied().rev().collect::<Vec<_>>();
            assert_eq!(probe.calls(), expected(&reversed, "revert"));
            assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 0);
            probe.assert_retained();
            assert_faulted(&engine);
            drop(engine);
            assert_eq!(probe.dropped.load(Ordering::SeqCst), 3);
        }
    }

    #[test]
    fn secondary_literal_interruption_during_push_preserves_both_sources() {
        let probe = Arc::new(Probe::default());
        let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
        let mut scope = engine
            .begin("push secondary", HistoryContextId::Global)
            .unwrap();
        scope.push(Command::new("prior", &probe)).unwrap();
        let error = scope
            .push(
                Command::new("failing", &probe)
                    .fail_apply(1, Failure::Applied)
                    .fail_revert(1, Failure::InterruptedLiteral),
            )
            .unwrap_err();
        assert_secondary(error);
        assert_eq!(
            probe.calls(),
            vec![
                String::from("prior.apply"),
                String::from("failing.apply"),
                String::from("failing.revert")
            ]
        );
        assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 0);
        probe.assert_retained();
        assert_faulted(&engine);
        drop(scope);
        probe.assert_retained();
        drop(engine);
        assert_eq!(probe.dropped.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn secondary_literal_interruption_during_undo_stops_recovery() {
        let probe = Arc::new(Probe::default());
        let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
        let mut scope = engine
            .begin("undo secondary", HistoryContextId::Global)
            .unwrap();
        scope.push(Command::new("first", &probe)).unwrap();
        scope
            .push(Command::new("middle", &probe).fail_revert(1, Failure::Unchanged))
            .unwrap();
        scope
            .push(Command::new("last", &probe).fail_apply(2, Failure::InterruptedLiteral))
            .unwrap();
        scope.commit().unwrap();
        probe.clear_calls();
        assert_secondary(engine.undo(HistoryContextId::Global).unwrap_err());
        assert_eq!(
            probe.calls(),
            vec![
                String::from("last.revert"),
                String::from("middle.revert"),
                String::from("last.apply")
            ]
        );
        assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 0);
        probe.assert_retained();
        assert_faulted(&engine);
        drop(engine);
        assert_eq!(probe.dropped.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn secondary_literal_interruption_during_redo_stops_recovery() {
        let probe = Arc::new(Probe::default());
        let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
        let mut scope = engine
            .begin("redo secondary", HistoryContextId::Global)
            .unwrap();
        scope
            .push(Command::new("first", &probe).fail_revert(2, Failure::InterruptedLiteral))
            .unwrap();
        scope
            .push(Command::new("middle", &probe).fail_apply(2, Failure::Unchanged))
            .unwrap();
        scope.push(Command::new("last", &probe)).unwrap();
        scope.commit().unwrap();
        engine.undo(HistoryContextId::Global).unwrap();
        probe.clear_calls();
        assert_secondary(engine.redo(HistoryContextId::Global).unwrap_err());
        assert_eq!(
            probe.calls(),
            vec![
                String::from("first.apply"),
                String::from("middle.apply"),
                String::from("first.revert")
            ]
        );
        assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 0);
        probe.assert_retained();
        assert_faulted(&engine);
        drop(engine);
        assert_eq!(probe.dropped.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn secondary_literal_interruption_during_cancel_stops_recovery() {
        let probe = Arc::new(Probe::default());
        let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
        let mut scope = engine
            .begin("cancel secondary", HistoryContextId::Global)
            .unwrap();
        scope.push(Command::new("first", &probe)).unwrap();
        scope
            .push(Command::new("middle", &probe).fail_revert(1, Failure::Unchanged))
            .unwrap();
        scope
            .push(Command::new("last", &probe).fail_apply(2, Failure::InterruptedLiteral))
            .unwrap();
        probe.clear_calls();
        assert_secondary(scope.cancel().unwrap_err());
        assert_eq!(
            probe.calls(),
            vec![
                String::from("last.revert"),
                String::from("middle.revert"),
                String::from("last.apply")
            ]
        );
        assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 0);
        probe.assert_retained();
        assert_faulted(&engine);
        drop(engine);
        assert_eq!(probe.dropped.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn interrupted_selection_restore_skips_history_and_cancel_recovery() {
        for operation in ["undo", "redo", "cancel"] {
            let probe = Arc::new(Probe::default());
            let mut context = Context::new(Arc::clone(&probe));
            context.interrupt_restore_at = Some(if operation == "redo" { 2 } else { 1 });
            let engine = EditorTransactionEngine::new(context);
            let mut scope = engine
                .begin("selection interrupted", HistoryContextId::Global)
                .unwrap();
            scope.push(Command::new("only", &probe)).unwrap();
            let error = if operation == "cancel" {
                probe.clear_calls();
                scope.cancel().unwrap_err()
            } else {
                scope.commit().unwrap();
                if operation == "redo" {
                    engine.undo(HistoryContextId::Global).unwrap();
                }
                probe.clear_calls();
                if operation == "redo" {
                    engine.redo(HistoryContextId::Global).unwrap_err()
                } else {
                    engine.undo(HistoryContextId::Global).unwrap_err()
                }
            };
            assert!(error.is_interrupted());
            assert_eq!(
                probe.calls(),
                expected(
                    &["only"],
                    if operation == "redo" {
                        "apply"
                    } else {
                        "revert"
                    }
                )
            );
            assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 1);
            probe.assert_retained();
            assert_faulted(&engine);
            drop(engine);
            assert_eq!(probe.dropped.load(Ordering::SeqCst), 1);
        }
    }

    #[test]
    fn interrupted_commit_after_apply_keeps_active_payload_without_compensation() {
        let probe = Arc::new(Probe::default());
        let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
        let mut scope = engine
            .begin("host interrupted", HistoryContextId::Global)
            .unwrap();
        scope.push(Command::new("only", &probe)).unwrap();
        let error = scope
            .commit_after_apply(|_| {
                Err(CommandExecutionError::interrupted(target("host lifecycle")).into_source())
            })
            .unwrap_err();
        assert!(error.is_interrupted());
        assert_eq!(probe.calls(), expected(&["only"], "apply"));
        assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 0);
        probe.assert_retained();
        assert_faulted(&engine);
        drop(engine);
        assert_eq!(probe.dropped.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn ordinary_applied_gateway_failure_still_recovers_prior_commands() {
        let probe = Arc::new(Probe::default());
        let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
        let mut scope = engine
            .begin("gateway failure", HistoryContextId::Global)
            .unwrap();
        scope.push(Command::new("prior", &probe)).unwrap();
        let error = scope
            .push(Command::new("gateway", &probe).fail_apply(1, Failure::GatewayApplied))
            .unwrap_err();
        assert!(!error.is_interrupted());
        assert!(matches!(error, EditCommandError::ExternalEffect { .. }));
        assert_eq!(
            probe.calls(),
            vec![
                String::from("prior.apply"),
                String::from("gateway.apply"),
                String::from("gateway.revert"),
                String::from("prior.revert")
            ]
        );
        assert_eq!(
            engine
                .with_context::<Context, _>(|context| context.value)
                .unwrap(),
            Some(0)
        );
        assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 1);
        assert_eq!(probe.finalized.load(Ordering::SeqCst), 2);
        assert_eq!(probe.dropped.load(Ordering::SeqCst), 2);
        drop(scope);
        engine
            .begin("still available", HistoryContextId::Global)
            .unwrap()
            .cancel()
            .unwrap();
    }

    #[test]
    fn nested_cancellation_marker_stops_even_with_ordinary_effect() {
        let probe = Arc::new(Probe::default());
        let engine = EditorTransactionEngine::new(Context::new(Arc::clone(&probe)));
        let mut scope = engine
            .begin("nested interruption", HistoryContextId::Global)
            .unwrap();
        let error = scope
            .push(Command::new("nested", &probe).fail_apply(1, Failure::NestedMarker))
            .unwrap_err();
        assert!(error.is_interrupted());
        assert!(matches!(error, EditCommandError::RollbackFailed { .. }));
        assert_eq!(probe.calls(), expected(&["nested"], "apply"));
        assert_eq!(probe.selection_restores.load(Ordering::SeqCst), 0);
        probe.assert_retained();
        assert_faulted(&engine);
        drop(scope);
        probe.assert_retained();
        drop(engine);
        assert_eq!(probe.dropped.load(Ordering::SeqCst), 1);
    }
}
