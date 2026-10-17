use std::sync::atomic::Ordering;

use crate::core::editing::engine::{
    EditCommandError, EditorTransactionEngine, HistoryContextId, HistorySaveMarkOutcome,
    TransactionId, TransactionJournalError,
};
use crate::core::editor_message::DocumentId;
use crate::core::play::{PlayInstanceId, WorldDomain};

use super::fixture::{finalized_counter, DeltaCommand, FixtureContext};

fn mark_current_saved(engine: &EditorTransactionEngine, history: HistoryContextId) {
    let token = engine.capture_save_token(history).unwrap();
    assert_eq!(
        engine.mark_saved_if_unchanged(history, token).unwrap(),
        HistorySaveMarkOutcome::Marked
    );
}

fn commit_delta(
    engine: &EditorTransactionEngine,
    history: HistoryContextId,
    label: &'static str,
    before: u8,
    after: i32,
) {
    let mut scope = engine.begin(label, history).unwrap();
    scope
        .push(DeltaCommand::new(label, before, after, finalized_counter()))
        .unwrap();
    scope.commit().unwrap();
}

#[test]
fn play_history_uses_its_world_route_and_remains_volatile() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let instance = PlayInstanceId::for_test(17);
    let history = HistoryContextId::PlaySession(instance);

    assert!(history.is_volatile());
    assert_eq!(history.world_domain(), WorldDomain::Play(instance));
    assert!(!engine.is_dirty(history).unwrap());
    let status = engine.history_status(history).unwrap();
    assert_eq!(status.len, 0);
    assert_eq!(status.saved_top, None);
    assert!(status.saved_top_reachable);
    assert!(!status.dirty);
    let mut scope = engine.begin("routed play edit", history).unwrap();
    scope
        .push(DeltaCommand::new(
            "routed play edit",
            1,
            5,
            finalized_counter(),
        ))
        .unwrap();
    scope.commit().unwrap();
    assert_eq!(engine.history_status(history).unwrap().len, 1);
    assert!(!engine.is_dirty(history).unwrap());
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.world_domain)
            .unwrap(),
        Some(WorldDomain::Play(instance))
    );
    assert!(engine.undo(history).unwrap());
    assert!(engine.redo(history).unwrap());
    assert!(matches!(
        engine.capture_save_token(history),
        Err(EditCommandError::VolatileHistoryPersistenceUnsupported {
            history: found,
            operation: "capture save token for",
        }) if found == history
    ));
    assert!(matches!(
        engine.journal_transaction(history, TransactionId::from_sequence(1)),
        Err(TransactionJournalError::VolatileHistory { history: found }) if found == history
    ));
    assert!(engine.discard_play_history(instance).unwrap());
    assert_eq!(engine.history_status(history).unwrap().len, 0);
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.world_domain)
            .unwrap(),
        Some(WorldDomain::Edit)
    );

    let empty_instance = PlayInstanceId::for_test(18);
    let empty_history = HistoryContextId::PlaySession(empty_instance);
    engine
        .begin("empty play edit", empty_history)
        .unwrap()
        .commit()
        .unwrap();
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.world_domain)
            .unwrap(),
        Some(WorldDomain::Play(empty_instance))
    );
    assert!(!engine.discard_play_history(empty_instance).unwrap());
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.world_domain)
            .unwrap(),
        Some(WorldDomain::Edit)
    );
}

#[test]
fn push_after_undo_truncates_redo_and_finalizes_it() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());

    for label in ["one", "two"] {
        let mut scope = engine.begin(label, HistoryContextId::Global).unwrap();
        scope
            .push(DeltaCommand::new(label, 1, 1, finalized.clone()))
            .unwrap();
        scope.commit().unwrap();
    }
    engine.undo(HistoryContextId::Global).unwrap();

    let mut replacement = engine
        .begin("replacement", HistoryContextId::Global)
        .unwrap();
    replacement
        .push(DeltaCommand::new("replacement", 2, 5, finalized.clone()))
        .unwrap();
    replacement.commit().unwrap();

    let status = engine.history_status(HistoryContextId::Global).unwrap();
    assert_eq!(status.len, 2);
    assert!(!status.can_redo);
    assert_eq!(finalized.load(Ordering::SeqCst), 1);
}

#[test]
fn history_status_is_narrow_and_details_are_explicitly_paged() {
    let engine = EditorTransactionEngine::with_capacity(FixtureContext::default(), 256).unwrap();

    for _ in 0..128 {
        commit_delta(
            &engine,
            HistoryContextId::Global,
            "paged history record",
            1,
            1,
        );
    }

    let status = engine.history_status(HistoryContextId::Global).unwrap();
    assert_eq!(status.len, 128);
    assert_eq!(status.generation, 128);
    assert_eq!(status.top.map(|transaction| transaction.raw()), Some(128));
    assert!(status.can_undo);
    assert!(!status.can_redo);
    assert!(status.dirty);

    let first = engine
        .history_details(HistoryContextId::Global, None, 1)
        .unwrap();
    assert_eq!(first.status(), status);
    assert_eq!(first.records().len(), 1);
    assert_eq!(first.records()[0].id.raw(), 1);
    let cursor = first.next_cursor().cloned().unwrap();

    let full_window = engine
        .history_details(HistoryContextId::Global, None, 128)
        .unwrap();
    assert_eq!(full_window.records().len(), 128);
    assert!(full_window.next_cursor().is_none());

    let remaining = engine
        .history_details(HistoryContextId::Global, Some(&cursor), 128)
        .unwrap();
    assert_eq!(remaining.records().len(), 127);
    assert_eq!(remaining.records()[0].id.raw(), 2);

    commit_delta(
        &engine,
        HistoryContextId::Global,
        "invalidate page cursor",
        1,
        1,
    );
    assert!(matches!(
        engine.history_details(HistoryContextId::Global, Some(&cursor), 17),
        Err(EditCommandError::HistoryPageCursorStale {
            history: HistoryContextId::Global,
            cursor_generation: 128,
            current_generation: 129,
        })
    ));
    assert!(matches!(
        engine.history_details(HistoryContextId::Global, None, 0),
        Err(EditCommandError::HistoryPageSizeOutOfRange { requested: 0, .. })
    ));
}

#[test]
fn history_page_cursor_is_bound_to_its_engine_and_history() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    let cursor = engine
        .history_details(HistoryContextId::Global, None, 1)
        .unwrap()
        .next_cursor()
        .cloned()
        .unwrap();
    let document = HistoryContextId::Document(DocumentId::new(42));

    assert!(matches!(
        engine.history_details(document, Some(&cursor), 1),
        Err(EditCommandError::HistoryPageCursorHistoryMismatch {
            cursor_history: HistoryContextId::Global,
            requested_history,
        }) if requested_history == document
    ));

    let foreign_engine = EditorTransactionEngine::new(FixtureContext::default());
    assert!(matches!(
        foreign_engine.history_details(HistoryContextId::Global, Some(&cursor), 1),
        Err(EditCommandError::HistoryPageCursorEngineMismatch)
    ));
}

#[test]
fn capacity_evicts_the_oldest_record_and_finalizes_it() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::with_capacity(FixtureContext::default(), 1).unwrap();

    for label in ["one", "two"] {
        let mut scope = engine.begin(label, HistoryContextId::Global).unwrap();
        scope
            .push(DeltaCommand::new(label, 1, 1, finalized.clone()))
            .unwrap();
        scope.commit().unwrap();
    }

    let status = engine.history_status(HistoryContextId::Global).unwrap();
    assert_eq!(status.len, 1);
    assert_eq!(status.top.map(|transaction| transaction.raw()), Some(2));
    assert_eq!(finalized.load(Ordering::SeqCst), 1);
}

#[test]
fn eviction_of_saved_first_record_keeps_the_saved_baseline_reachable() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::with_capacity(FixtureContext::default(), 1).unwrap();

    let mut saved = engine.begin("saved", HistoryContextId::Global).unwrap();
    saved
        .push(DeltaCommand::new("saved", 1, 1, finalized.clone()))
        .unwrap();
    saved.commit().unwrap();
    mark_current_saved(&engine, HistoryContextId::Global);

    let mut current = engine.begin("current", HistoryContextId::Global).unwrap();
    current
        .push(DeltaCommand::new("current", 2, 2, finalized))
        .unwrap();
    current.commit().unwrap();

    let after_eviction = engine.history_status(HistoryContextId::Global).unwrap();
    assert_eq!(after_eviction.saved_top, None);
    assert!(after_eviction.saved_top_reachable);
    assert!(engine.is_dirty(HistoryContextId::Global).unwrap());

    engine.undo(HistoryContextId::Global).unwrap();
    assert!(!engine.is_dirty(HistoryContextId::Global).unwrap());
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(1)
    );
}

#[test]
fn saved_top_is_the_dirty_state_authority_even_after_branching() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    assert!(!engine.is_dirty(HistoryContextId::Global).unwrap());

    let mut first = engine.begin("one", HistoryContextId::Global).unwrap();
    first
        .push(DeltaCommand::new("one", 1, 1, finalized.clone()))
        .unwrap();
    first.commit().unwrap();
    assert!(engine.is_dirty(HistoryContextId::Global).unwrap());

    mark_current_saved(&engine, HistoryContextId::Global);
    assert!(!engine.is_dirty(HistoryContextId::Global).unwrap());
    engine.undo(HistoryContextId::Global).unwrap();
    assert!(engine.is_dirty(HistoryContextId::Global).unwrap());

    let mut branch = engine.begin("branch", HistoryContextId::Global).unwrap();
    branch
        .push(DeltaCommand::new("branch", 2, 3, finalized))
        .unwrap();
    branch.commit().unwrap();
    assert!(engine.is_dirty(HistoryContextId::Global).unwrap());
    assert!(
        !engine
            .history_status(HistoryContextId::Global)
            .unwrap()
            .saved_top_reachable
    );
}

#[test]
fn save_token_rejects_commit_between_capture_and_completion() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    let token = engine.capture_save_token(HistoryContextId::Global).unwrap();

    commit_delta(&engine, HistoryContextId::Global, "second", 1, 2);

    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, token),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
    assert!(engine.is_dirty(HistoryContextId::Global).unwrap());
}

#[test]
fn save_token_rejects_same_top_branch_replacement() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "original", 0, 1);
    let token = engine.capture_save_token(HistoryContextId::Global).unwrap();

    engine.undo(HistoryContextId::Global).unwrap();
    commit_delta(&engine, HistoryContextId::Global, "replacement", 0, 2);
    assert_eq!(
        engine
            .history_status(HistoryContextId::Global)
            .unwrap()
            .top
            .map(|transaction| transaction.raw()),
        Some(2)
    );

    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, token),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
}

#[test]
fn save_token_rejects_undo_and_redo_between_capture_and_completion() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    commit_delta(&engine, HistoryContextId::Global, "second", 1, 2);
    let before_undo = engine.capture_save_token(HistoryContextId::Global).unwrap();
    engine.undo(HistoryContextId::Global).unwrap();
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, before_undo),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));

    let before_redo = engine.capture_save_token(HistoryContextId::Global).unwrap();
    engine.redo(HistoryContextId::Global).unwrap();
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, before_redo),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
}

#[test]
fn empty_history_token_is_typed_and_invalidated_by_first_commit() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let token = engine.capture_save_token(HistoryContextId::Global).unwrap();
    assert_eq!(
        engine
            .mark_saved_if_unchanged(HistoryContextId::Global, token.clone())
            .unwrap(),
        HistorySaveMarkOutcome::AlreadyMarked
    );

    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, token),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
}

#[test]
fn save_token_rejects_cross_document_use() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let first = HistoryContextId::Document(DocumentId::new(1));
    let second = HistoryContextId::Document(DocumentId::new(2));
    let token = engine.capture_save_token(first).unwrap();

    assert!(matches!(
        engine.mark_saved_if_unchanged(second, token),
        Err(EditCommandError::SaveTokenHistoryMismatch {
            token_history,
            requested_history,
        }) if token_history == first && requested_history == second
    ));
}

#[test]
fn invalid_save_token_does_not_flush_an_open_operation_group() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let token_history = HistoryContextId::Document(DocumentId::new(31));
    let token = engine.capture_save_token(token_history).unwrap();
    let group = engine
        .execute_operation(
            "grouped edit",
            HistoryContextId::Global,
            Some("fixture.save-token-side-effect"),
            crate::core::editing::engine::MergeMode::All,
            Box::new(DeltaCommand::new("grouped edit", 1, 1, finalized_counter())),
        )
        .unwrap();

    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, token),
        Err(EditCommandError::SaveTokenHistoryMismatch { .. })
    ));
    assert_eq!(
        engine.flush_operation_group().unwrap(),
        Some(group.transaction_id)
    );
}

#[test]
fn save_token_rejects_cross_engine_use() {
    let first = EditorTransactionEngine::new(FixtureContext::default());
    let second = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&first, HistoryContextId::Global, "same", 0, 1);
    commit_delta(&second, HistoryContextId::Global, "same", 0, 1);
    let foreign = first.capture_save_token(HistoryContextId::Global).unwrap();

    assert!(matches!(
        second.mark_saved_if_unchanged(HistoryContextId::Global, foreign),
        Err(EditCommandError::SaveTokenEngineMismatch)
    ));
    assert!(second.is_dirty(HistoryContextId::Global).unwrap());
}

#[test]
fn save_token_capture_and_completion_reject_active_transaction_scopes() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "saved candidate", 0, 1);
    let token = engine.capture_save_token(HistoryContextId::Global).unwrap();

    let mut active = engine.begin("active", HistoryContextId::Global).unwrap();
    active
        .push(DeltaCommand::new("active", 1, 2, finalized_counter()))
        .unwrap();
    assert!(matches!(
        engine.capture_save_token(HistoryContextId::Global),
        Err(EditCommandError::SaveTokenActiveTransaction {
            operation: "capture save token",
            ..
        })
    ));
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, token.clone()),
        Err(EditCommandError::SaveTokenActiveTransaction {
            operation: "mark saved if unchanged",
            ..
        })
    ));

    active.cancel().unwrap();
    assert_eq!(
        engine
            .mark_saved_if_unchanged(HistoryContextId::Global, token)
            .unwrap(),
        HistorySaveMarkOutcome::Marked
    );
}

#[test]
fn repeated_save_completion_is_reported_without_moving_the_baseline() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    let token = engine.capture_save_token(HistoryContextId::Global).unwrap();

    assert_eq!(
        engine
            .mark_saved_if_unchanged(HistoryContextId::Global, token.clone())
            .unwrap(),
        HistorySaveMarkOutcome::Marked
    );
    assert_eq!(
        engine
            .mark_saved_if_unchanged(HistoryContextId::Global, token)
            .unwrap(),
        HistorySaveMarkOutcome::AlreadyMarked
    );
    assert!(!engine.is_dirty(HistoryContextId::Global).unwrap());
}

#[test]
fn save_token_is_invalidated_by_capacity_eviction_and_history_clear() {
    let engine = EditorTransactionEngine::with_capacity(FixtureContext::default(), 1).unwrap();
    commit_delta(&engine, HistoryContextId::Global, "first", 0, 1);
    let before_eviction = engine.capture_save_token(HistoryContextId::Global).unwrap();
    commit_delta(&engine, HistoryContextId::Global, "second", 1, 2);
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, before_eviction),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));

    let before_clear = engine.capture_save_token(HistoryContextId::Global).unwrap();
    let mut transition = engine
        .begin_exclusive_transition("clear history for save-token test")
        .unwrap();
    transition
        .clear_history_and_context::<FixtureContext>(
            HistoryContextId::Global,
            "FixtureContext",
            |_| Ok(()),
        )
        .unwrap();
    drop(transition);
    assert!(matches!(
        engine.mark_saved_if_unchanged(HistoryContextId::Global, before_clear),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
}

#[test]
fn multi_document_tokens_complete_independently() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let first = HistoryContextId::Document(DocumentId::new(11));
    let second = HistoryContextId::Document(DocumentId::new(12));
    commit_delta(&engine, first, "first", 0, 1);
    commit_delta(&engine, second, "second", 1, 2);
    let first_token = engine.capture_save_token(first).unwrap();
    let second_token = engine.capture_save_token(second).unwrap();

    commit_delta(&engine, second, "second changed", 2, 3);

    assert_eq!(
        engine.mark_saved_if_unchanged(first, first_token).unwrap(),
        HistorySaveMarkOutcome::Marked
    );
    assert!(matches!(
        engine.mark_saved_if_unchanged(second, second_token),
        Err(EditCommandError::HistoryChangedDuringSave { .. })
    ));
    assert!(!engine.is_dirty(first).unwrap());
    assert!(engine.is_dirty(second).unwrap());
}

use crate::core::editing::engine::{
    CommandExecutionError, CommandRetentionPlanV1, EditCommand, EditContext,
    HistoryRetentionBudgetV1, HistoryRetentionDomainV1, MergeMode, MergeOutcome,
    RetentionMaterializationV1, RetentionPhaseV1,
};
use std::any::Any;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::{Arc, Condvar, Mutex};
use zircon_runtime_interface::retained_memory::{
    ImmutableRetainedOwnerV1, RetainedAccountingModelV1, RetainedBoundBuilderV1, RetainedOwnerV1,
    RetentionContractError,
};

struct RetentionDropGateV1 {
    entered: std::sync::atomic::AtomicBool,
    released: Mutex<bool>,
    ready: Condvar,
    committed_seen: AtomicU64,
    retiring_seen: AtomicU64,
    domain: HistoryRetentionDomainV1,
}
impl RetentionDropGateV1 {
    fn release(&self) {
        *self.released.lock().unwrap() = true;
        self.ready.notify_all();
    }
    fn wait_until_entered(&self) -> bool {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut released = self.released.lock().unwrap();
        while !self.entered.load(Ordering::SeqCst) {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            if remaining.is_zero() {
                return false;
            }
            let result = self.ready.wait_timeout(released, remaining).unwrap();
            released = result.0;
            if result.1.timed_out() && !self.entered.load(Ordering::SeqCst) {
                return false;
            }
        }
        true
    }
}
impl RetainedOwnerV1 for RetentionDropGateV1 {
    fn retained_bound_v1(
        &self,
        _: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        // The shared domain accounts its own ledger separately. This fixture's
        // remaining state has no transitive heap and never grows while retained.
        Ok(())
    }
}
unsafe impl ImmutableRetainedOwnerV1 for RetentionDropGateV1 {}

struct RetainedBytesCommandV1 {
    bytes: Vec<u8>,
    applications: Arc<AtomicUsize>,
    gate: Option<Arc<RetentionDropGateV1>>,
    fail: bool,
    merge: bool,
}
impl RetainedBytesCommandV1 {
    fn new(capacity: usize, applications: Arc<AtomicUsize>) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
            applications,
            gate: None,
            fail: false,
            merge: false,
        }
    }
}
impl RetainedOwnerV1 for RetainedBytesCommandV1 {
    fn retained_bound_v1(
        &self,
        m: &mut RetainedBoundBuilderV1,
    ) -> Result<(), RetentionContractError> {
        self.bytes.retained_bound_v1(m)?;
        self.applications.retained_bound_v1(m)?;
        self.gate.retained_bound_v1(m)
    }
}
impl Drop for RetainedBytesCommandV1 {
    fn drop(&mut self) {
        if let Some(gate) = &self.gate {
            let snapshot = gate.domain.snapshot().unwrap();
            gate.committed_seen
                .store(snapshot.committed_bytes, Ordering::SeqCst);
            gate.retiring_seen
                .store(snapshot.retiring_bytes, Ordering::SeqCst);
            gate.entered.store(true, Ordering::SeqCst);
            gate.ready.notify_all();
            let mut released = gate.released.lock().unwrap();
            while !*released {
                released = gate.ready.wait(released).unwrap();
            }
        }
    }
}
impl EditCommand for RetainedBytesCommandV1 {
    fn retention_plan_v1(
        &self,
        context: &dyn EditContext,
        phase: RetentionPhaseV1,
        model: RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<CommandRetentionPlanV1, EditCommandError> {
        CommandRetentionPlanV1::for_owner(self, context, phase, model, limit)
    }
    fn prepare_retention_v1(
        &mut self,
        _: &mut dyn EditContext,
        phase: RetentionPhaseV1,
        permit: &mut RetentionMaterializationV1,
    ) -> Result<(), CommandExecutionError> {
        permit
            .consume(self, phase)
            .map_err(CommandExecutionError::unchanged)
    }
    fn label(&self) -> &str {
        "retained byte fixture"
    }
    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        if self.fail {
            return Err(CommandExecutionError::unchanged(
                EditCommandError::InvariantViolation {
                    invariant: "intentional unchanged refusal",
                },
            ));
        }
        self.applications.fetch_add(1, Ordering::SeqCst);
        context
            .as_any_mut()
            .downcast_mut::<FixtureContext>()
            .unwrap()
            .value += 1;
        Ok(())
    }
    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        context
            .as_any_mut()
            .downcast_mut::<FixtureContext>()
            .unwrap()
            .value -= 1;
        Ok(())
    }
    fn try_merge(&mut self, next: &dyn EditCommand) -> MergeOutcome {
        if self.merge
            && next
                .as_any()
                .downcast_ref::<Self>()
                .is_some_and(|next| next.merge)
        {
            MergeOutcome::Merged
        } else {
            MergeOutcome::Reject
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn retention_test_domain_v1(record: u64, history: u64, global: u64) -> HistoryRetentionDomainV1 {
    HistoryRetentionDomainV1::try_new(HistoryRetentionBudgetV1 {
        record_bytes: record,
        history_bytes: history,
        committed_bytes: global,
        staging_bytes: 8 * 1024 * 1024,
    })
    .unwrap()
}
fn retention_test_engine_v1(
    capacity: usize,
    domain: HistoryRetentionDomainV1,
) -> EditorTransactionEngine {
    EditorTransactionEngine::with_retention_domain_v1(FixtureContext::default(), capacity, domain)
        .unwrap()
}
fn commit_retained_v1(
    engine: &EditorTransactionEngine,
    history: HistoryContextId,
    bytes: usize,
) -> TransactionId {
    let mut scope = engine.begin("retained record", history).unwrap();
    scope
        .push(RetainedBytesCommandV1::new(
            bytes,
            Arc::new(AtomicUsize::new(0)),
        ))
        .unwrap();
    scope.commit().unwrap();
    engine.history_status(history).unwrap().top.unwrap()
}

#[test]
fn oversized_command_is_rejected_before_apply_and_preserves_history() {
    let domain = retention_test_domain_v1(32 * 1024, 256 * 1024, 4 * 1024 * 1024);
    let engine = retention_test_engine_v1(128, domain);
    let history = HistoryContextId::Global;
    commit_retained_v1(&engine, history, 1024);
    mark_current_saved(&engine, history);
    let before = engine.history_status(history).unwrap();
    let applied = Arc::new(AtomicUsize::new(0));
    let mut scope = engine.begin("too large", history).unwrap();
    let refused = scope.push(RetainedBytesCommandV1::new(64 * 1024, applied.clone()));
    let rollback = scope.cancel();
    assert!(matches!(refused, Err(EditCommandError::Retention(_))));
    rollback.unwrap();
    assert_eq!(applied.load(Ordering::SeqCst), 0);
    assert_eq!(engine.history_status(history).unwrap(), before);
}

#[test]
fn global_quota_does_not_evict_other_document_history() {
    let domain = retention_test_domain_v1(128 * 1024, 128 * 1024, 1024 * 1024);
    let engine = retention_test_engine_v1(8, domain.clone());
    let first = HistoryContextId::Document(DocumentId::new(201));
    commit_retained_v1(&engine, first, 24 * 1024);
    let first_before = engine.history_status(first).unwrap();
    let applied = Arc::new(AtomicUsize::new(0));
    let mut refused = false;
    for index in 0..128 {
        let second = HistoryContextId::Document(DocumentId::new(202 + index));
        let result = engine.begin("another document", second);
        let Ok(mut scope) = result else {
            refused = true;
            break;
        };
        match scope.push(RetainedBytesCommandV1::new(96 * 1024, applied.clone())) {
            Ok(()) => {
                scope.commit().unwrap();
            }
            Err(EditCommandError::Retention(_)) => {
                scope.cancel().unwrap();
                refused = true;
                break;
            }
            Err(error) => panic!("unexpected refusal: {error}"),
        }
    }
    assert!(
        refused,
        "finite shared metadata/payload budget must eventually refuse"
    );
    assert_eq!(engine.history_status(first).unwrap(), first_before);
    assert!(domain.snapshot().unwrap().committed_bytes <= domain.budget().committed_bytes);
}

#[test]
fn committed_payload_stays_charged_until_retiring_drop_finishes() {
    let domain = retention_test_domain_v1(128 * 1024, 128 * 1024, 4 * 1024 * 1024);
    let gate = Arc::new(RetentionDropGateV1 {
        entered: std::sync::atomic::AtomicBool::new(false),
        released: Mutex::new(false),
        ready: Condvar::new(),
        committed_seen: AtomicU64::new(0),
        retiring_seen: AtomicU64::new(0),
        domain: domain.clone(),
    });
    let engine = retention_test_engine_v1(1, domain.clone());
    let mut first = RetainedBytesCommandV1::new(4096, Arc::new(AtomicUsize::new(0)));
    first.gate = Some(gate.clone());
    let mut scope = engine
        .begin("held payload", HistoryContextId::Global)
        .unwrap();
    scope.push(first).unwrap();
    scope.commit().unwrap();
    let before = domain.snapshot().unwrap();
    let worker = std::thread::spawn(move || {
        commit_retained_v1(&engine, HistoryContextId::Global, 4096);
        engine
    });
    let entered = gate.wait_until_entered();
    let while_dropping = domain.snapshot();
    gate.release();
    let result = worker.join();
    assert!(entered);
    let while_dropping = while_dropping.unwrap();
    let engine = result.unwrap();
    assert!(while_dropping.retiring_bytes >= 4096);
    assert!(
        while_dropping.committed_bytes + while_dropping.retiring_bytes >= before.committed_bytes
    );
    assert_eq!(domain.snapshot().unwrap().retiring_bytes, 0);
    assert_eq!(
        engine.history_status(HistoryContextId::Global).unwrap().len,
        1
    );
}

#[test]
fn failed_apply_keeps_planned_eviction_and_saved_top_unchanged() {
    let domain = retention_test_domain_v1(128 * 1024, 128 * 1024, 4 * 1024 * 1024);
    let engine = retention_test_engine_v1(1, domain);
    let history = HistoryContextId::Global;
    commit_retained_v1(&engine, history, 4096);
    mark_current_saved(&engine, history);
    let before = engine.history_status(history).unwrap();
    let mut command = RetainedBytesCommandV1::new(4096, Arc::new(AtomicUsize::new(0)));
    command.fail = true;
    let mut scope = engine.begin("fails before mutation", history).unwrap();
    let failure = scope.push(command);
    drop(scope); // unchanged apply failure already closed its scope
    assert!(failure.is_err());
    assert_eq!(engine.history_status(history).unwrap(), before);
}

#[test]
fn nested_scopes_and_operation_groups_transfer_one_retention_reservation() {
    let domain = retention_test_domain_v1(128 * 1024, 256 * 1024, 4 * 1024 * 1024);
    let engine = retention_test_engine_v1(128, domain.clone());
    let history = HistoryContextId::Global;
    let mut outer = engine.begin("outer", history).unwrap();
    outer
        .push(RetainedBytesCommandV1::new(
            1024,
            Arc::new(AtomicUsize::new(0)),
        ))
        .unwrap();
    let mut inner = engine.begin("inner", history).unwrap();
    inner
        .push(RetainedBytesCommandV1::new(
            2048,
            Arc::new(AtomicUsize::new(0)),
        ))
        .unwrap();
    inner.commit().unwrap();
    outer.commit().unwrap();
    assert_eq!(engine.history_status(history).unwrap().len, 1);
    engine
        .execute_operation(
            "group",
            history,
            Some("one gesture"),
            MergeMode::All,
            Box::new(RetainedBytesCommandV1::new(
                1024,
                Arc::new(AtomicUsize::new(0)),
            )),
        )
        .unwrap();
    let during = domain.snapshot().unwrap();
    assert!(during.staging_bytes > 0);
    engine.flush_operation_group().unwrap();
    let after = domain.snapshot().unwrap();
    assert_eq!(after.staging_bytes, 0);
    assert_eq!(after.promised_bytes, 0);
    assert_eq!(engine.history_status(history).unwrap().len, 2);
}

#[test]
fn merged_commands_keep_charge_through_discarded_payload_drop() {
    let domain = retention_test_domain_v1(128 * 1024, 128 * 1024, 4 * 1024 * 1024);
    let gate = Arc::new(RetentionDropGateV1 {
        entered: std::sync::atomic::AtomicBool::new(false),
        released: Mutex::new(false),
        ready: Condvar::new(),
        committed_seen: AtomicU64::new(0),
        retiring_seen: AtomicU64::new(0),
        domain: domain.clone(),
    });
    // Pre-release avoids a blocking callback; its snapshot still executes in
    // the exact discarded-command Drop and can acquire the unlocked ledger.
    gate.release();
    let engine = retention_test_engine_v1(128, domain.clone());
    let mut first = RetainedBytesCommandV1::new(4096, Arc::new(AtomicUsize::new(0)));
    first.merge = true;
    engine
        .execute_operation(
            "merge",
            HistoryContextId::Global,
            Some("merge gesture"),
            MergeMode::All,
            Box::new(first),
        )
        .unwrap();
    let mut next = RetainedBytesCommandV1::new(8192, Arc::new(AtomicUsize::new(0)));
    next.merge = true;
    next.gate = Some(gate.clone());
    engine
        .execute_operation(
            "merge",
            HistoryContextId::Global,
            Some("merge gesture"),
            MergeMode::All,
            Box::new(next),
        )
        .unwrap();
    let dropped = gate.entered.load(Ordering::SeqCst);
    let during = domain.snapshot().unwrap();
    engine.flush_operation_group().unwrap();
    assert!(dropped);
    assert!(during.staging_bytes >= 8192);
    assert_eq!(domain.snapshot().unwrap().staging_bytes, 0);
    assert_eq!(
        engine.history_status(HistoryContextId::Global).unwrap().len,
        1
    );
}

#[test]
fn pinned_records_and_undo_barriers_prevent_budget_retirement() {
    let domain = retention_test_domain_v1(128 * 1024, 128 * 1024, 4 * 1024 * 1024);
    let engine = retention_test_engine_v1(1, domain);
    let history = HistoryContextId::Global;
    let first = commit_retained_v1(&engine, history, 1024);
    let pin = engine.pin_history_record_v1(history, first).unwrap();
    let before = engine.history_status(history).unwrap();
    let refused = engine.execute_operation(
        "cannot retire pinned",
        history,
        None,
        MergeMode::Disable,
        Box::new(RetainedBytesCommandV1::new(
            1024,
            Arc::new(AtomicUsize::new(0)),
        )),
    );
    assert!(refused.is_err());
    assert_eq!(engine.history_status(history).unwrap(), before);
    drop(pin);
    let barrier = engine.history_undo_barrier_v1(history, first).unwrap();
    assert!(!engine.history_status(history).unwrap().can_undo);
    assert!(!engine.undo(history).unwrap());
    drop(barrier);
    assert!(engine.undo(history).unwrap());
}

#[test]
fn byte_eviction_preserves_saved_top_and_redo_branch_reachability() {
    let domain = retention_test_domain_v1(48 * 1024, 64 * 1024, 4 * 1024 * 1024);
    let engine = retention_test_engine_v1(128, domain);
    let history = HistoryContextId::Global;
    commit_retained_v1(&engine, history, 20 * 1024);
    mark_current_saved(&engine, history);
    commit_retained_v1(&engine, history, 20 * 1024);
    let old_count = engine.history_status(history).unwrap().len;
    commit_retained_v1(&engine, history, 20 * 1024);
    let evicted = engine.history_status(history).unwrap();
    assert!(
        evicted.len < old_count + 1,
        "byte cap applies before count 128"
    );
    assert!(evicted.dirty);
    assert!(engine.undo(history).unwrap());
    commit_retained_v1(&engine, history, 1024);
    let branched = engine.history_status(history).unwrap();
    assert!(!branched.can_redo);
    assert!(branched.dirty);
}
