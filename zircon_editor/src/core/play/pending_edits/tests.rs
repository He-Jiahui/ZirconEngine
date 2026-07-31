use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use crate::core::editing::operation::{
    DeferredOperationInvocation, OperationCommand, OperationCommandFactory,
    OperationCommandFactoryError, OperationCommandFactoryRegistration, PendingEditRetention,
    PendingEditRetentionError,
};
use crate::core::editor_message::DocumentId;
use crate::core::editor_operation::EditorOperationInvocation;

use super::super::{
    PlayEditResolutionError, PlayEditRoute, PlayEditTarget, PlayKind, PlaySessionController,
    PlaySessionError, PlayStartRequest,
};
use super::*;

fn invocation(name: &str) -> EditorOperationInvocation {
    EditorOperationInvocation::parse(format!("editor.test.{name}"))
        .expect("test operation path should be valid")
}

fn deferred(
    invocation: EditorOperationInvocation,
    retention: PendingEditRetention,
) -> DeferredOperationInvocation {
    OperationCommandFactoryRegistration::new(
        invocation.operation_id.clone(),
        "Pending edit test",
        Arc::new(PendingEditFixtureFactory),
    )
    .with_pending_edit_retention(retention)
    .defer(invocation)
    .expect("registration must bind the invocation operation")
}

fn lossless(name: &str) -> DeferredOperationInvocation {
    deferred(invocation(name), PendingEditRetention::Lossless)
}

struct PendingEditFixtureFactory;

impl OperationCommandFactory for PendingEditFixtureFactory {
    fn create(
        &self,
        _invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        unreachable!("deferred retention tests do not execute the operation")
    }
}

#[test]
fn latest_retention_coalesces_only_the_same_target_and_operation() {
    let queue = PendingEditQueue::default();
    let operation = invocation("rename");
    let first = queue
        .enqueue(
            PlayEditTarget::EditWorkspace,
            deferred(
                operation
                    .clone()
                    .with_arguments(serde_json::json!({ "value": "old" }))
                    .with_operation_group("rename-before"),
                PendingEditRetention::latest(),
            ),
        )
        .unwrap();
    let replacement = queue
        .enqueue(
            PlayEditTarget::EditWorkspace,
            deferred(
                operation
                    .clone()
                    .with_arguments(serde_json::json!({ "value": "new" }))
                    .with_operation_group("rename-after"),
                PendingEditRetention::latest(),
            ),
        )
        .unwrap();
    let other_target = queue
        .enqueue(
            PlayEditTarget::EditDocument(DocumentId::new(11)),
            deferred(operation, PendingEditRetention::latest()),
        )
        .unwrap();

    assert_eq!(replacement.id, first.id);
    assert!(replacement.coalesced);
    assert_ne!(other_target.id, first.id);
    let page = queue.page(None, 8);
    assert_eq!(page.entries.len(), 2);
    assert_eq!(page.entries[0].operation_id, "editor.test.rename");
    let mut applied_groups = Vec::new();
    let applied = queue.apply_with_budget(PendingEditApplyBudget::unlimited(), |intent| {
        applied_groups.push(intent.invocation.operation_group.clone());
        Ok::<(), ()>(())
    });
    assert_eq!(applied.applied, vec![first.id, other_target.id]);
    assert_eq!(applied_groups, vec![Some("rename-after".to_string()), None]);
}

#[test]
fn lossless_retention_preserves_fifo_order_and_retry_authority() {
    let queue = PendingEditQueue::default();
    let first = queue
        .enqueue(PlayEditTarget::EditWorkspace, lossless("first"))
        .unwrap()
        .id;
    let failed = queue
        .enqueue(PlayEditTarget::EditWorkspace, lossless("failed"))
        .unwrap()
        .id;
    let third = queue
        .enqueue(PlayEditTarget::EditWorkspace, lossless("third"))
        .unwrap()
        .id;

    let first_turn = queue.apply_with_budget(PendingEditApplyBudget::unlimited(), |intent| {
        if intent.id == failed {
            Err("target missing")
        } else {
            Ok(())
        }
    });
    assert_eq!(first_turn.applied, vec![first, third]);
    assert_eq!(first_turn.failures[0].intent.id, failed);

    let second_turn =
        queue.apply_with_budget(PendingEditApplyBudget::unlimited(), |_| Ok::<(), ()>(()));
    assert_eq!(second_turn.applied, vec![failed]);
    assert_eq!(second_turn.remaining.pending_count, 0);
}

#[test]
fn bounded_retention_evicts_only_its_typed_cohort() {
    let queue = PendingEditQueue::default();
    let policy = || PendingEditRetention::bounded(2, 1_024, Duration::from_secs(5)).unwrap();
    let operation = invocation("drag");
    let first = queue
        .enqueue(
            PlayEditTarget::EditWorkspace,
            deferred(operation.clone(), policy()),
        )
        .unwrap();
    let second = queue
        .enqueue(
            PlayEditTarget::EditWorkspace,
            deferred(operation.clone(), policy()),
        )
        .unwrap();
    let third = queue
        .enqueue(PlayEditTarget::EditWorkspace, deferred(operation, policy()))
        .unwrap();
    let other_operation = queue
        .enqueue(
            PlayEditTarget::EditWorkspace,
            deferred(invocation("other-drag"), policy()),
        )
        .unwrap();

    assert_eq!(third.evicted_ids, vec![first.id]);
    assert!(other_operation.evicted_ids.is_empty());
    assert_eq!(
        queue
            .page(None, 8)
            .entries
            .into_iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        vec![second.id, third.id, other_operation.id]
    );
}

#[test]
fn bounded_retention_rejects_excess_payload_without_dropping_retained_work() {
    let queue = PendingEditQueue::default();
    let policy = PendingEditRetention::bounded(2, 1, Duration::from_secs(5)).unwrap();

    let error = queue
        .enqueue(
            PlayEditTarget::EditWorkspace,
            deferred(invocation("large"), policy),
        )
        .unwrap_err();

    assert!(matches!(
        error,
        PendingEditQueueError::RetentionPayloadLimitExceeded { .. }
    ));
    assert!(queue.is_empty());
}

#[test]
fn bounded_retention_rejects_new_work_after_its_own_age_limit() {
    let queue = PendingEditQueue::default();
    let policy = || PendingEditRetention::bounded(2, 1_024, Duration::from_millis(1)).unwrap();
    let operation = invocation("drag");
    queue
        .enqueue(
            PlayEditTarget::EditWorkspace,
            deferred(operation.clone(), policy()),
        )
        .unwrap();
    thread::sleep(Duration::from_millis(5));

    assert!(matches!(
        queue.enqueue(PlayEditTarget::EditWorkspace, deferred(operation, policy()),),
        Err(PendingEditQueueError::RetentionAgeExceeded { .. })
    ));
    assert_eq!(queue.summary().pending_count, 1);
}

#[test]
fn lossless_admission_respects_global_limits_without_dropping_another_intent() {
    let queue = PendingEditQueue::new(PendingEditQueueLimits {
        max_entries: 1,
        max_payload_bytes: usize::MAX,
        max_oldest_age: Duration::from_secs(60),
    });
    let first = queue
        .enqueue(PlayEditTarget::EditWorkspace, lossless("first"))
        .unwrap();

    assert_eq!(
        queue.enqueue(PlayEditTarget::EditWorkspace, lossless("second")),
        Err(PendingEditQueueError::EntryLimitReached)
    );
    assert_eq!(queue.page(None, 8).entries[0].id, first.id);
}

#[test]
fn bounded_policy_rejects_zero_limits_before_play_routing() {
    assert_eq!(
        PendingEditRetention::bounded(0, 1, Duration::from_secs(1)),
        Err(PendingEditRetentionError::ZeroBoundedEntries)
    );
    assert_eq!(
        PendingEditRetention::bounded(1, 0, Duration::from_secs(1)),
        Err(PendingEditRetentionError::ZeroBoundedPayloadBytes)
    );
    assert_eq!(
        PendingEditRetention::bounded(1, 1, Duration::ZERO),
        Err(PendingEditRetentionError::ZeroBoundedAge)
    );
}

#[test]
fn queued_route_uses_the_operation_registration_retention_contract() {
    let controller = PlaySessionController::new();
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();

    let route = controller
        .route_edit(
            PlayEditTarget::EditWorkspace,
            deferred(invocation("rename"), PendingEditRetention::latest()),
        )
        .unwrap();

    assert!(matches!(
        route,
        PlayEditRoute::Queued {
            coalesced: false,
            ..
        }
    ));
}

#[test]
fn pending_decision_blocks_the_next_play_start_until_queue_resolution() {
    let controller = PlaySessionController::new();
    controller
        .request_play(
            PlayStartRequest::immediate(PlayKind::Play, None)
                .with_running_document(DocumentId::new(10)),
        )
        .unwrap();
    controller
        .route_edit(
            PlayEditTarget::EditDocument(DocumentId::new(11)),
            lossless("rename"),
        )
        .unwrap();
    let stopped = controller.request_stop().unwrap();

    assert_eq!(stopped.pending_edit_prompt.unwrap().pending_count, 1);
    assert!(matches!(
        controller.request_play(PlayStartRequest::immediate(PlayKind::Play, None)),
        Err(PlaySessionError::PendingEditDecisionRequired { pending_count: 1 })
    ));
}

#[test]
fn playing_cannot_resolve_pending_edits() {
    let controller = PlaySessionController::new();
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    controller
        .route_edit(PlayEditTarget::EditWorkspace, lossless("queued"))
        .unwrap();

    assert_eq!(
        controller.apply_pending_edits(PendingEditApplyBudget::unlimited(), |_| Ok::<(), ()>(())),
        Err(PlayEditResolutionError::PlayActive)
    );
    assert_eq!(controller.pending_edits_summary().pending_count, 1);
}

#[test]
fn resolution_in_progress_blocks_play_start() {
    let controller = Arc::new(PlaySessionController::new());
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    controller
        .route_edit(PlayEditTarget::EditWorkspace, lossless("queued"))
        .unwrap();
    controller.request_stop().unwrap();

    let callback_entered = Arc::new(Barrier::new(2));
    let callback_release = Arc::new(Barrier::new(2));
    let worker_controller = Arc::clone(&controller);
    let worker_entered = Arc::clone(&callback_entered);
    let worker_release = Arc::clone(&callback_release);
    let worker = thread::spawn(move || {
        worker_controller
            .apply_pending_edits(PendingEditApplyBudget::unlimited(), |_| {
                worker_entered.wait();
                worker_release.wait();
                Ok::<(), ()>(())
            })
            .unwrap()
    });

    callback_entered.wait();
    assert!(matches!(
        controller.request_play(PlayStartRequest::immediate(PlayKind::Play, None)),
        Err(PlaySessionError::PendingEditResolutionInProgress)
    ));
    callback_release.wait();
    assert_eq!(worker.join().unwrap().applied.len(), 1);
}
