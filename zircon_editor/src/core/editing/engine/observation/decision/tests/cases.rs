use std::any::Any;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;

use super::super::super::*;
use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditContext, EditWorldRoute, MergeOutcome,
    SelectionSnapshot,
};
use crate::core::play::{PlayInstanceId, WorldDomain};

struct Context(Arc<AtomicU64>);
impl EditContext for Context {
    fn capture_world_route(&self, domain: WorldDomain) -> Result<EditWorldRoute, EditCommandError> {
        if self.0.load(Ordering::SeqCst) == u64::MAX {
            return Err(EditCommandError::WorldRouteUnavailable {
                world_domain: domain,
            });
        }
        Ok(EditWorldRoute::runtime(
            domain,
            crate::core::gateway::GatewaySessionIdentity::detached()
                .with_gateway_generation(self.0.load(Ordering::SeqCst)),
        ))
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
struct Add {
    value: Arc<AtomicI64>,
    amount: i64,
}
impl EditCommand for Add {
    fn label(&self) -> &str {
        "add"
    }
    fn apply(&mut self, _: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        self.value.fetch_add(self.amount, Ordering::SeqCst);
        Ok(())
    }
    fn revert(&mut self, _: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        self.value.fetch_sub(self.amount, Ordering::SeqCst);
        Ok(())
    }
    fn try_merge(&mut self, next: &dyn EditCommand) -> MergeOutcome {
        let Some(next) = next.as_any().downcast_ref::<Self>() else {
            return MergeOutcome::Reject;
        };
        self.amount += next.amount;
        MergeOutcome::Merged
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
fn command(value: &Arc<AtomicI64>) -> Add {
    Add {
        value: Arc::clone(value),
        amount: 1,
    }
}
fn engine() -> EditorTransactionEngine {
    EditorTransactionEngine::new(Context(Arc::new(AtomicU64::new(0))))
}
fn update(
    engine: &EditorTransactionEngine,
    history: HistoryContextId,
    mode: MergeMode,
    value: &Arc<AtomicI64>,
) -> TransactionId {
    engine
        .execute_operation(
            "interaction",
            history,
            Some("interaction"),
            mode,
            Box::new(command(value)),
        )
        .unwrap()
        .transaction_id
}

#[test]
fn decision_token_observes_pending_without_committing_and_detects_same_group_append() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    assert_eq!(engine.dirty_history_decision_token(h).unwrap(), None);
    let id = update(&engine, h, MergeMode::Disable, &value);
    let first = engine.dirty_history_decision_token(h).unwrap().unwrap();
    assert_eq!(
        engine.dirty_history_decision_token(h).unwrap(),
        Some(first.clone())
    );
    assert_eq!(engine.history_status(h).unwrap().len, 0);
    assert_eq!(update(&engine, h, MergeMode::Disable, &value), id);
    assert_ne!(engine.dirty_history_decision_token(h).unwrap(), Some(first));
    assert_eq!(engine.history_generation_snapshot(h).unwrap(), 0);
    engine.cancel(id).unwrap();
    assert_eq!(value.load(Ordering::SeqCst), 0);
    assert_eq!(engine.dirty_history_decision_token(h).unwrap(), None);
}
#[test]
fn decision_token_detects_merged_apply_even_when_command_count_and_transaction_stay_equal() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let id = update(&engine, h, MergeMode::Ends, &value);
    let first = engine.dirty_history_decision_token(h).unwrap();
    assert_eq!(update(&engine, h, MergeMode::Ends, &value), id);
    assert_eq!(engine.lock_state().active[0].commands.len(), 1);
    assert_ne!(engine.dirty_history_decision_token(h).unwrap(), first);
    engine.cancel(id).unwrap();
    assert_eq!(value.load(Ordering::SeqCst), 0);
}
#[test]
fn decision_token_detects_commit_and_cancel_then_reapply() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let id = update(&engine, h, MergeMode::Disable, &value);
    let pending = engine.dirty_history_decision_token(h).unwrap();
    engine.cancel(id).unwrap();
    update(&engine, h, MergeMode::Disable, &value);
    let successor = engine.dirty_history_decision_token(h).unwrap();
    assert_ne!(successor, pending);
    engine.flush_operation_group().unwrap();
    assert_ne!(engine.dirty_history_decision_token(h).unwrap(), successor);
    assert_eq!(engine.history_status(h).unwrap().len, 1);
}
#[test]
fn decision_token_detects_nested_cancel_with_parent_still_dirty() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let mut parent = engine.begin("parent", h).unwrap();
    parent.push(command(&value)).unwrap();
    let mut child = engine.begin("child", h).unwrap();
    child.push(command(&value)).unwrap();
    let before = engine.dirty_history_decision_token(h).unwrap();
    child.cancel().unwrap();
    assert_ne!(engine.dirty_history_decision_token(h).unwrap(), before);
    assert_eq!(value.load(Ordering::SeqCst), 1);
    parent.cancel().unwrap();
}
#[test]
fn decision_token_preserves_engine_history_and_world_identity() {
    let first = engine();
    let second = engine();
    let value = Arc::new(AtomicI64::new(0));
    let h = HistoryContextId::Global;
    update(&first, h, MergeMode::Disable, &value);
    update(&second, h, MergeMode::Disable, &value);
    assert_ne!(
        first.dirty_history_decision_token(h).unwrap(),
        second.dirty_history_decision_token(h).unwrap()
    );
    let token = first.dirty_history_decision_token(h).unwrap().unwrap();
    let mut changed = token.clone();
    changed.history = HistoryContextId::Document(DocumentId::new(63));
    assert_ne!(token, changed);
    let mut changed = token.clone();
    changed.route = EditWorldRoute::logical(WorldDomain::Play(PlayInstanceId::for_test(63)));
    assert_ne!(token, changed);
    let mut changed = token.clone();
    changed.pending.as_mut().unwrap().route = changed.route.clone();
    assert_ne!(token, changed);
}
#[test]
fn decision_token_preserves_busy_fault_and_route_errors() {
    let unavailable = Arc::new(AtomicU64::new(0));
    let engine = EditorTransactionEngine::new(Context(Arc::clone(&unavailable)));
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let id = update(&engine, h, MergeMode::Disable, &value);
    engine.lock_state().operation = Some("push command");
    assert!(matches!(
        engine.dirty_history_decision_token(h),
        Err(EditCommandError::EngineBusy { .. })
    ));
    engine.lock_state().operation = None;
    engine.lock_state().faulted = true;
    assert!(matches!(
        engine.dirty_history_decision_token(h),
        Err(EditCommandError::EngineFaulted { .. })
    ));
    engine.lock_state().faulted = false;
    unavailable.store(u64::MAX, Ordering::SeqCst);
    assert!(matches!(
        engine.dirty_history_decision_token(h),
        Err(EditCommandError::WorldRouteUnavailable { .. })
    ));
    unavailable.store(0, Ordering::SeqCst);
    assert_eq!(update(&engine, h, MergeMode::Disable, &value), id);
    engine.cancel(id).unwrap();
}
#[test]
fn exhausted_apply_revision_rejects_before_world_mutation() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let mut scope = engine.begin("exhausted", h).unwrap();
    engine.lock_state().applied_command_revision = u64::MAX;
    assert!(matches!(
        scope.push(command(&value)),
        Err(EditCommandError::AppliedCommandRevisionExhausted)
    ));
    assert_eq!(value.load(Ordering::SeqCst), 0);
    scope.cancel().unwrap();
}

#[test]
fn decision_token_detects_nested_commit_and_live_gateway_replacement() {
    let gateway_generation = Arc::new(AtomicU64::new(0));
    let engine = EditorTransactionEngine::new(Context(Arc::clone(&gateway_generation)));
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let mut parent = engine.begin("parent", h).unwrap();
    parent.push(command(&value)).unwrap();
    let mut child = engine.begin("child", h).unwrap();
    child.push(command(&value)).unwrap();
    let before = engine.dirty_history_decision_token(h).unwrap();
    child.commit().unwrap();
    assert_ne!(engine.dirty_history_decision_token(h).unwrap(), before);
    let before = engine.dirty_history_decision_token(h).unwrap();
    gateway_generation.store(1, Ordering::SeqCst);
    assert_ne!(engine.dirty_history_decision_token(h).unwrap(), before);
    gateway_generation.store(0, Ordering::SeqCst);
    parent.cancel().unwrap();
}
#[test]
fn exhausted_cancel_revision_preserves_pending_world_and_scope() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let id = update(&engine, h, MergeMode::Disable, &value);
    engine.lock_state().applied_command_revision = u64::MAX;
    assert!(matches!(
        engine.cancel(id),
        Err(EditCommandError::AppliedCommandRevisionExhausted)
    ));
    assert_eq!(value.load(Ordering::SeqCst), 1);
    assert!(engine.has_active_scope(id));
    engine.lock_state().applied_command_revision = 1;
    engine.cancel(id).unwrap();
}

struct Reject;
impl EditCommand for Reject {
    fn label(&self) -> &str {
        "reject"
    }
    fn apply(&mut self, _: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Err(CommandExecutionError::unchanged(
            EditCommandError::InvariantViolation {
                invariant: "rejected apply fixture",
            },
        ))
    }
    fn revert(&mut self, _: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        panic!("unchanged rejection must not be reverted")
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[test]
fn decision_token_detects_automatic_child_cancel_after_failed_push() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let mut parent = engine.begin("parent", h).unwrap();
    parent.push(command(&value)).unwrap();
    let mut child = engine.begin("child", h).unwrap();
    child.push(command(&value)).unwrap();
    let before = engine.dirty_history_decision_token(h).unwrap();
    assert!(child.push(Reject).is_err());
    assert_eq!(value.load(Ordering::SeqCst), 1);
    assert_ne!(engine.dirty_history_decision_token(h).unwrap(), before);
    parent.cancel().unwrap();
}
#[test]
fn decision_token_detects_automatic_child_cancel_after_rejected_after_apply() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let mut parent = engine.begin("parent", h).unwrap();
    parent.push(command(&value)).unwrap();
    let mut child = engine.begin("child", h).unwrap();
    child.push(command(&value)).unwrap();
    let before = engine.dirty_history_decision_token(h).unwrap();
    assert!(child
        .commit_after_apply(|_| Err(EditCommandError::InvariantViolation {
            invariant: "rejected after-apply fixture"
        }))
        .is_err());
    assert_eq!(value.load(Ordering::SeqCst), 1);
    assert_ne!(engine.dirty_history_decision_token(h).unwrap(), before);
    parent.cancel().unwrap();
}

#[test]
fn authorized_pending_transition_checks_before_flush_and_retains_one_gate_through_clear() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    update(&engine, h, MergeMode::Ends, &value);
    let token = engine.dirty_history_decision_token(h).unwrap().unwrap();
    let mut transition = engine
        .begin_history_transition::<Context>(
            "authorized replacement",
            h,
            Some(&token),
            "Context",
            |_, _| Ok(()),
        )
        .unwrap();
    assert_eq!(transition.history_status(h).len, 1);
    assert!(matches!(
        engine.execute_operation(
            "competing",
            h,
            Some("interaction"),
            MergeMode::Ends,
            Box::new(command(&value))
        ),
        Err(EditCommandError::EngineBusy { .. })
    ));
    transition
        .clear_history_and_context::<Context>(h, "Context", |_| Ok(()))
        .unwrap();
    assert!(!transition.history_status(h).dirty);
    drop(transition);
    assert!(!engine.is_dirty(h).unwrap());
}
#[test]
fn stale_pending_transition_rejects_append_and_merge_without_flushing_or_mutating() {
    for mode in [MergeMode::Disable, MergeMode::Ends] {
        let engine = engine();
        let h = HistoryContextId::Global;
        let value = Arc::new(AtomicI64::new(0));
        let id = update(&engine, h, mode, &value);
        let token = engine.dirty_history_decision_token(h).unwrap().unwrap();
        assert_eq!(update(&engine, h, mode, &value), id);
        let before = engine.dirty_history_decision_token(h).unwrap();
        assert!(matches!(
            engine.begin_history_transition::<Context>(
                "stale replacement",
                h,
                Some(&token),
                "Context",
                |_, _| Ok(())
            ),
            Err(EditCommandError::HistoryDecisionChanged { .. })
        ));
        assert_eq!(engine.dirty_history_decision_token(h).unwrap(), before);
        assert_eq!(engine.history_status(h).unwrap().len, 0);
        assert_eq!(engine.history_generation_snapshot(h).unwrap(), 0);
        assert_eq!(value.load(Ordering::SeqCst), 2);
        assert_eq!(update(&engine, h, mode, &value), id);
        engine.cancel(id).unwrap();
    }
}
#[test]
fn history_transition_rejects_canceled_dirty_and_missing_authorization_before_flush() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let id = update(&engine, h, MergeMode::Disable, &value);
    let token = engine.dirty_history_decision_token(h).unwrap().unwrap();
    assert!(matches!(
        engine
            .begin_history_transition::<Context>("unauthorized", h, None, "Context", |_, _| Ok(())),
        Err(EditCommandError::HistoryDecisionChanged { .. })
    ));
    assert_eq!(engine.history_status(h).unwrap().len, 0);
    engine.cancel(id).unwrap();
    assert!(matches!(
        engine.begin_history_transition::<Context>(
            "canceled authorization",
            h,
            Some(&token),
            "Context",
            |_, _| Ok(())
        ),
        Err(EditCommandError::HistoryDecisionChanged { .. })
    ));
    let clean = engine
        .begin_history_transition::<Context>("clean replacement", h, None, "Context", |_, _| Ok(()))
        .unwrap();
    drop(clean);
}
#[test]
fn authorized_history_transition_rejects_stale_route_busy_fault_and_manual_scope() {
    let route = Arc::new(AtomicU64::new(0));
    let engine = EditorTransactionEngine::new(Context(Arc::clone(&route)));
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let id = update(&engine, h, MergeMode::Disable, &value);
    let token = engine.dirty_history_decision_token(h).unwrap().unwrap();
    route.store(1, Ordering::SeqCst);
    assert!(matches!(
        engine.begin_history_transition::<Context>(
            "stale route",
            h,
            Some(&token),
            "Context",
            |_, _| Ok(())
        ),
        Err(EditCommandError::HistoryDecisionChanged { .. })
    ));
    assert_eq!(engine.history_generation_snapshot(h).unwrap(), 0);
    route.store(0, Ordering::SeqCst);
    engine.lock_state().operation = Some("push command");
    assert!(matches!(
        engine
            .begin_history_transition::<Context>("busy", h, Some(&token), "Context", |_, _| Ok(())),
        Err(EditCommandError::EngineBusy { .. })
    ));
    engine.lock_state().operation = None;
    engine.lock_state().faulted = true;
    assert!(matches!(
        engine.begin_history_transition::<Context>(
            "faulted",
            h,
            Some(&token),
            "Context",
            |_, _| Ok(())
        ),
        Err(EditCommandError::EngineFaulted { .. })
    ));
    engine.lock_state().faulted = false;
    engine.cancel(id).unwrap();
    let mut scope = engine.begin("manual", h).unwrap();
    scope.push(command(&value)).unwrap();
    let token = engine.dirty_history_decision_token(h).unwrap().unwrap();
    assert!(matches!(
        engine.begin_history_transition::<Context>(
            "manual owner",
            h,
            Some(&token),
            "Context",
            |_, _| Ok(())
        ),
        Err(EditCommandError::InvariantViolation { .. })
    ));
    assert_eq!(value.load(Ordering::SeqCst), 1);
    scope.cancel().unwrap();
}
#[test]
fn pending_flush_events_publish_only_after_exclusive_reservation_is_released() {
    struct Sink(Arc<AtomicU64>);
    impl TransactionEventSink for Sink {
        fn publish(&self, event: TransactionEvent) -> TransactionEventDelivery {
            if event.kind == TransactionEventKind::Committed {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
            TransactionEventDelivery::Delivered
        }
    }
    let committed = Arc::new(AtomicU64::new(0));
    let engine = EditorTransactionEngine::with_event_sink(
        Context(Arc::new(AtomicU64::new(0))),
        Arc::new(Sink(Arc::clone(&committed))),
    );
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    update(&engine, h, MergeMode::Disable, &value);
    let token = engine.dirty_history_decision_token(h).unwrap().unwrap();
    let transition = engine
        .begin_history_transition::<Context>(
            "deferred notification",
            h,
            Some(&token),
            "Context",
            |_, _| Ok(()),
        )
        .unwrap();
    assert_eq!(committed.load(Ordering::SeqCst), 0);
    assert!(matches!(
        engine.begin("competing", h),
        Err(EditCommandError::EngineBusy { .. })
    ));
    drop(transition);
    assert_eq!(committed.load(Ordering::SeqCst), 1);
}
#[test]
fn authorized_flush_failure_preserves_pending_scope_and_decision() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let id = update(&engine, h, MergeMode::Disable, &value);
    engine.set_history_generation_for_test(h, u64::MAX);
    let token = engine.dirty_history_decision_token(h).unwrap().unwrap();
    assert!(matches!(
        engine.begin_history_transition::<Context>(
            "exhausted flush",
            h,
            Some(&token),
            "Context",
            |_, _| Ok(())
        ),
        Err(EditCommandError::HistoryGenerationExhausted { .. })
    ));
    assert_eq!(engine.dirty_history_decision_token(h).unwrap(), Some(token));
    assert_eq!(value.load(Ordering::SeqCst), 1);
    engine.set_history_generation_for_test(h, 0);
    engine.cancel(id).unwrap();
}

#[test]
fn fresh_decision_does_not_authorize_a_pending_scope_pinned_to_a_stale_route() {
    let route = Arc::new(AtomicU64::new(0));
    let engine = EditorTransactionEngine::new(Context(Arc::clone(&route)));
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let id = update(&engine, h, MergeMode::Disable, &value);
    route.store(1, Ordering::SeqCst);
    let current = engine.dirty_history_decision_token(h).unwrap().unwrap();
    assert!(matches!(
        engine.begin_history_transition::<Context>(
            "fresh but stale pending route",
            h,
            Some(&current),
            "Context",
            |_, _| Ok(())
        ),
        Err(EditCommandError::WorldRouteStale { .. })
    ));
    assert_eq!(engine.history_generation_snapshot(h).unwrap(), 0);
    assert_eq!(engine.history_status(h).unwrap().len, 0);
    route.store(0, Ordering::SeqCst);
    engine.cancel(id).unwrap();
}

#[test]
fn history_clear_preflight_rejects_second_counter_exhaustion_without_committing_pending() {
    for dirty_exhaustion in [false, true] {
        let engine = engine();
        let h = HistoryContextId::Global;
        let value = Arc::new(AtomicI64::new(0));
        let id = update(&engine, h, MergeMode::Disable, &value);
        if dirty_exhaustion {
            engine.set_dirty_generation_for_test(u64::MAX - 1);
        } else {
            engine.set_history_generation_for_test(h, u64::MAX - 1);
        }
        let token = engine.dirty_history_decision_token(h).unwrap().unwrap();
        let dirty_before = engine.dirty_states_since(None).unwrap();
        assert!(matches!(
            engine.begin_history_transition::<Context>(
                "two-step exhausted",
                h,
                Some(&token),
                "Context",
                |_, _| Ok(())
            ),
            Err(EditCommandError::HistoryGenerationExhausted { .. }
                | EditCommandError::HistoryDirtyGenerationExhausted)
        ));
        assert_eq!(engine.dirty_history_decision_token(h).unwrap(), Some(token));
        assert_eq!(engine.history_status(h).unwrap().len, 0);
        assert_eq!(
            engine
                .dirty_states_since(Some(dirty_before.cursor()))
                .unwrap()
                .kind(),
            HistoryDirtyBatchKind::Unchanged
        );
        assert_eq!(value.load(Ordering::SeqCst), 1);
        assert_eq!(update(&engine, h, MergeMode::Disable, &value), id);
        engine.set_dirty_generation_for_test(0);
        engine.set_history_generation_for_test(h, 0);
        engine.cancel(id).unwrap();
    }
}
#[test]
fn history_clear_preflight_accepts_exact_pending_two_steps_and_clean_single_step() {
    let h = HistoryContextId::Global;
    for pending in [false, true] {
        let engine = engine();
        let value = Arc::new(AtomicI64::new(0));
        if pending {
            update(&engine, h, MergeMode::Disable, &value);
        }
        let start = u64::MAX - if pending { 2 } else { 1 };
        engine.set_history_generation_for_test(h, start);
        engine.set_dirty_generation_for_test(start);
        let token = engine.dirty_history_decision_token(h).unwrap();
        let mut transition = engine
            .begin_history_transition::<Context>(
                "exact budget",
                h,
                token.as_ref(),
                "Context",
                |_, _| Ok(()),
            )
            .unwrap();
        transition
            .clear_history_and_context::<Context>(h, "Context", |_| Ok(()))
            .unwrap();
        assert_eq!(transition.history_status(h).generation, u64::MAX);
        assert!(!transition.history_status(h).dirty);
        drop(transition);
        assert_eq!(engine.dirty_generation_for_test(), u64::MAX);
    }
}
#[test]
fn history_clear_preflight_budgets_distinct_group_and_target_histories_separately() {
    let target = HistoryContextId::Global;
    let group = HistoryContextId::Document(DocumentId::new(64));
    let engine = engine();
    let value = Arc::new(AtomicI64::new(0));
    let id = update(&engine, group, MergeMode::Disable, &value);
    engine.set_history_generation_for_test(target, u64::MAX - 1);
    engine.set_history_generation_for_test(group, u64::MAX - 1);
    engine.set_dirty_generation_for_test(u64::MAX - 1);
    let group_before = engine.dirty_history_decision_token(group).unwrap();
    assert!(matches!(
        engine.begin_history_transition::<Context>(
            "distinct journal exhausted",
            target,
            None,
            "Context",
            |_, _| Ok(())
        ),
        Err(EditCommandError::HistoryDirtyGenerationExhausted)
    ));
    assert_eq!(
        engine.dirty_history_decision_token(group).unwrap(),
        group_before
    );
    assert_eq!(engine.history_status(group).unwrap().len, 0);
    engine.set_dirty_generation_for_test(u64::MAX - 2);
    let mut transition = engine
        .begin_history_transition::<Context>(
            "distinct exact budget",
            target,
            None,
            "Context",
            |_, _| Ok(()),
        )
        .unwrap();
    transition
        .clear_history_and_context::<Context>(target, "Context", |_| Ok(()))
        .unwrap();
    assert_eq!(transition.history_status(target).generation, u64::MAX);
    assert_eq!(transition.history_status(group).generation, u64::MAX);
    assert_eq!(transition.history_status(group).top, Some(id));
}
#[test]
fn typed_preparation_failure_and_context_mismatch_leave_pending_decision_unchanged() {
    let engine = engine();
    let h = HistoryContextId::Global;
    let value = Arc::new(AtomicI64::new(0));
    let id = update(&engine, h, MergeMode::Disable, &value);
    let token = engine.dirty_history_decision_token(h).unwrap().unwrap();
    assert!(matches!(
        engine.begin_history_transition::<Context>(
            "prepare rejected",
            h,
            Some(&token),
            "Context",
            |_, _| Err(EditCommandError::SelectionGenerationExhausted)
        ),
        Err(EditCommandError::SelectionGenerationExhausted)
    ));
    assert!(matches!(
        engine.begin_history_transition::<u64>("wrong context", h, Some(&token), "u64", |_, _| Ok(
            ()
        )),
        Err(EditCommandError::ContextTypeMismatch { .. })
    ));
    assert_eq!(engine.dirty_history_decision_token(h).unwrap(), Some(token));
    assert_eq!(engine.history_generation_snapshot(h).unwrap(), 0);
    assert_eq!(engine.history_status(h).unwrap().len, 0);
    assert_eq!(value.load(Ordering::SeqCst), 1);
    engine.cancel(id).unwrap();
}
