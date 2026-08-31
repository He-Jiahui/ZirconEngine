use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use crate::core::editing::operation::{
    DeferredOperationInvocation, EditOperationTarget, OperationCommand, OperationCommandFactory,
    OperationCommandFactoryError, OperationCommandFactoryRegistration, PendingEditRetention,
    PendingEditRetentionError,
};
use crate::core::editor_message::DocumentId;
use crate::core::editor_operation::EditorOperationInvocation;

use super::super::{
    PlayEditResolutionError, PlayEditRoute, PlayEditRouteError, PlayKind, PlaySessionController,
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
        EditOperationTarget::EditWorkspace,
        Arc::new(PendingEditFixtureFactory),
    )
    .with_pending_edit_retention(retention)
    .defer(invocation)
    .expect("registration must bind the invocation operation")
}

fn lossless(name: &str) -> DeferredOperationInvocation {
    deferred(invocation(name), PendingEditRetention::Lossless)
}

#[test]
fn optimization_batch_20260826b_editor07_pending_edit_page_reserves_bounded_capacity() {
    let source = include_str!("queue.rs");
    let page = source
        .split_once("pub fn page(")
        .expect("pending edit page implementation should exist")
        .1
        .split_once("pub fn decision_prompt(")
        .expect("pending edit page implementation should remain bounded")
        .0;

    assert!(page.contains("let page_capacity = candidates.clone().count().min(limit)"));
    assert!(page.contains("Vec::with_capacity(page_capacity)"));
    assert!(page.contains("candidates.by_ref().take(limit)"));
    assert!(!page.contains("collect::<Vec<_>>()"));
}

#[test]
fn optimization_batch_20260826b_editor07_pending_edit_page_preserves_cursor_order() {
    let queue = PendingEditQueue::default();
    let ids = (0..130)
        .map(|index| {
            queue
                .enqueue(
                    EditOperationTarget::EditWorkspace,
                    lossless(&format!("page_{index}")),
                )
                .unwrap()
                .id
        })
        .collect::<Vec<_>>();

    let first = queue.page(None, usize::MAX);
    let second = queue.page(first.next_cursor, usize::MAX);

    assert_eq!(first.entries.len(), 128);
    assert_eq!(first.entries.first().unwrap().id, ids[0]);
    assert_eq!(first.entries.last().unwrap().id, ids[127]);
    assert_eq!(first.next_cursor.unwrap().id(), ids[127]);
    assert_eq!(
        second
            .entries
            .iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        ids[128..]
    );
    assert!(second.next_cursor.is_none());
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
            EditOperationTarget::EditWorkspace,
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
            EditOperationTarget::EditWorkspace,
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
            EditOperationTarget::EditDocument(DocumentId::new(11)),
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
        .enqueue(EditOperationTarget::EditWorkspace, lossless("first"))
        .unwrap()
        .id;
    let failed = queue
        .enqueue(EditOperationTarget::EditWorkspace, lossless("failed"))
        .unwrap()
        .id;
    let third = queue
        .enqueue(EditOperationTarget::EditWorkspace, lossless("third"))
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
fn budgeted_apply_leaves_unattempted_intents_for_the_next_resolution_turn() {
    let queue = PendingEditQueue::default();
    let first = queue
        .enqueue(EditOperationTarget::EditWorkspace, lossless("first"))
        .unwrap()
        .id;
    let second = queue
        .enqueue(EditOperationTarget::EditWorkspace, lossless("second"))
        .unwrap()
        .id;

    let first_turn = queue.apply_with_budget(PendingEditApplyBudget::new(1, Duration::MAX), |_| {
        Ok::<(), ()>(())
    });
    assert_eq!(first_turn.applied, vec![first]);
    assert!(first_turn.budget_exhausted);
    assert_eq!(first_turn.remaining.pending_count, 1);

    let second_turn =
        queue.apply_with_budget(PendingEditApplyBudget::unlimited(), |_| Ok::<(), ()>(()));
    assert_eq!(second_turn.applied, vec![second]);
    assert_eq!(second_turn.remaining.pending_count, 0);
}

#[test]
fn apply_freezes_the_original_batch_and_keeps_in_flight_capacity_reserved() {
    let queue = Arc::new(PendingEditQueue::new(PendingEditQueueLimits {
        max_entries: 2,
        max_payload_bytes: usize::MAX,
        max_oldest_age: Duration::from_secs(60),
    }));
    queue
        .enqueue(EditOperationTarget::EditWorkspace, lossless("first"))
        .unwrap();
    queue
        .enqueue(EditOperationTarget::EditWorkspace, lossless("second"))
        .unwrap();

    let callback_queue = Arc::clone(&queue);
    let report = queue.apply_with_budget(PendingEditApplyBudget::unlimited(), |intent| {
        if intent.invocation.operation_id.as_str() == "editor.test.first" {
            assert!(matches!(
                callback_queue.enqueue(EditOperationTarget::EditWorkspace, lossless("third")),
                Err(PendingEditQueueError::EntryLimitReached)
            ));
        }
        Ok::<(), ()>(())
    });

    assert_eq!(report.applied.len(), 2);
    assert_eq!(report.remaining.pending_count, 0);
}

#[test]
fn apply_callback_panic_requeues_the_in_flight_intent() {
    let queue = PendingEditQueue::default();
    let id = queue
        .enqueue(EditOperationTarget::EditWorkspace, lossless("panic"))
        .unwrap()
        .id;

    let panic = catch_unwind(AssertUnwindSafe(|| {
        queue.apply_with_budget(PendingEditApplyBudget::unlimited(), |_| {
            panic!("apply callback failed")
        });
    }));
    assert!(panic.is_err());
    assert_eq!(queue.summary().pending_count, 1);

    let report = queue.apply_with_budget(PendingEditApplyBudget::unlimited(), |intent| {
        assert_eq!(intent.id, id);
        Ok::<(), ()>(())
    });
    assert_eq!(report.applied, vec![id]);
    assert_eq!(report.remaining.pending_count, 0);
}

#[test]
fn bounded_retention_evicts_only_its_typed_cohort() {
    let queue = PendingEditQueue::default();
    let policy = || PendingEditRetention::bounded(2, 1_024, Duration::from_secs(5)).unwrap();
    let operation = invocation("drag");
    let first = queue
        .enqueue(
            EditOperationTarget::EditWorkspace,
            deferred(operation.clone(), policy()),
        )
        .unwrap();
    let second = queue
        .enqueue(
            EditOperationTarget::EditWorkspace,
            deferred(operation.clone(), policy()),
        )
        .unwrap();
    let third = queue
        .enqueue(
            EditOperationTarget::EditWorkspace,
            deferred(operation, policy()),
        )
        .unwrap();
    let other_operation = queue
        .enqueue(
            EditOperationTarget::EditWorkspace,
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
            EditOperationTarget::EditWorkspace,
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
            EditOperationTarget::EditWorkspace,
            deferred(operation.clone(), policy()),
        )
        .unwrap();
    thread::sleep(Duration::from_millis(5));

    let error = queue
        .enqueue(
            EditOperationTarget::EditWorkspace,
            deferred(operation, policy()),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        PendingEditQueueError::RetentionAgeExceeded {
            operation_id,
            max_oldest_age,
            ..
        } if operation_id == "drag" && max_oldest_age == Duration::from_millis(1)
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
        .enqueue(EditOperationTarget::EditWorkspace, lossless("first"))
        .unwrap();

    assert_eq!(
        queue.enqueue(EditOperationTarget::EditWorkspace, lossless("second")),
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
            EditOperationTarget::EditWorkspace,
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
fn route_rejects_a_target_that_does_not_match_the_operation_registration() {
    let controller = PlaySessionController::new();
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    let deferred = deferred(
        invocation("registered_document"),
        PendingEditRetention::Lossless,
    );

    assert_eq!(
        controller.route_edit(EditOperationTarget::PlayDomain, deferred),
        Err(PlayEditRouteError::TargetMismatch {
            requested: EditOperationTarget::PlayDomain,
            registered: EditOperationTarget::EditWorkspace,
        })
    );
}

#[test]
fn queued_route_surfaces_declared_bounded_evictions_to_its_caller() {
    let controller = PlaySessionController::new();
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    let policy = || PendingEditRetention::bounded(1, 1_024, Duration::from_secs(5)).unwrap();
    let operation = invocation("drag");

    let first = controller
        .route_edit(
            EditOperationTarget::EditWorkspace,
            deferred(operation.clone(), policy()),
        )
        .unwrap();
    let first_id = match first {
        PlayEditRoute::Queued {
            id,
            ref evicted_ids,
            ..
        } => {
            assert!(evicted_ids.is_empty());
            id
        }
        route => panic!("expected queued route, got {route:?}"),
    };

    let replacement = controller
        .route_edit(
            EditOperationTarget::EditWorkspace,
            deferred(operation, policy()),
        )
        .unwrap();
    assert!(matches!(
        replacement,
        PlayEditRoute::Queued {
            evicted_ids,
            ..
        } if evicted_ids == vec![first_id]
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
            EditOperationTarget::EditDocument(DocumentId::new(11)),
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
        .route_edit(EditOperationTarget::EditWorkspace, lossless("queued"))
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
        .route_edit(EditOperationTarget::EditWorkspace, lossless("queued"))
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

#[test]
fn concurrent_resolver_cannot_republish_a_prompt_while_the_queue_is_resolving() {
    let controller = Arc::new(PlaySessionController::new());
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    controller
        .route_edit(EditOperationTarget::EditWorkspace, lossless("queued"))
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
    assert_eq!(
        controller.apply_pending_edits(PendingEditApplyBudget::unlimited(), |_| Ok::<(), ()>(())),
        Err(PlayEditResolutionError::ResolutionInProgress)
    );
    assert!(
        controller.pending_edit_decision_prompt().is_none(),
        "a rejected resolver must not republish a stale prompt while the owner can still clear it"
    );

    callback_release.wait();
    assert_eq!(worker.join().unwrap().applied.len(), 1);
    assert!(controller.pending_edit_decision_prompt().is_none());
}

#[test]
fn concurrent_resolver_reveals_retry_prompt_after_owner_finishes_with_failure() {
    let controller = Arc::new(PlaySessionController::new());
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    controller
        .route_edit(EditOperationTarget::EditWorkspace, lossless("queued"))
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
                Err::<(), _>("apply deliberately failed")
            })
            .unwrap()
    });

    callback_entered.wait();
    assert_eq!(
        controller.apply_pending_edits(PendingEditApplyBudget::unlimited(), |_| Ok::<(), ()>(())),
        Err(PlayEditResolutionError::ResolutionInProgress)
    );
    assert_eq!(
        controller.route_edit(EditOperationTarget::EditWorkspace, lossless("later")),
        Err(PlayEditRouteError::PendingResolutionInProgress)
    );
    assert_eq!(
        controller.request_play(PlayStartRequest::immediate(PlayKind::Play, None)),
        Err(PlaySessionError::PendingEditResolutionInProgress)
    );
    assert!(controller.pending_edit_decision_prompt().is_none());

    callback_release.wait();
    assert_eq!(worker.join().unwrap().failures.len(), 1);
    assert_eq!(
        controller
            .pending_edit_decision_prompt()
            .expect("retry intent should become visible once the owner releases the barrier")
            .pending_count,
        1
    );
}

#[test]
fn decision_publication_blocks_a_new_resolver_until_the_current_prompt_is_published() {
    let controller = Arc::new(PlaySessionController::new());
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    controller
        .route_edit(EditOperationTarget::EditWorkspace, lossless("queued"))
        .unwrap();
    controller.request_stop().unwrap();

    let publish_entered = Arc::new(Barrier::new(2));
    let publish_release = Arc::new(Barrier::new(2));
    let worker_controller = Arc::clone(&controller);
    let worker_entered = Arc::clone(&publish_entered);
    let worker_release = Arc::clone(&publish_release);
    let publisher = thread::spawn(move || {
        worker_controller
            .with_pending_edit_decision_prompt(|prompt| {
                assert_eq!(prompt.pending_count, 1);
                worker_entered.wait();
                worker_release.wait();
                Ok::<(), ()>(())
            })
            .unwrap()
    });

    publish_entered.wait();
    assert_eq!(
        controller.apply_pending_edits(PendingEditApplyBudget::unlimited(), |_| Ok::<(), ()>(())),
        Err(PlayEditResolutionError::ResolutionInProgress)
    );
    assert_eq!(
        controller.route_edit(EditOperationTarget::EditWorkspace, lossless("later")),
        Err(PlayEditRouteError::PendingResolutionInProgress)
    );
    assert_eq!(
        controller.request_play(PlayStartRequest::immediate(PlayKind::Play, None)),
        Err(PlaySessionError::PendingEditResolutionInProgress)
    );
    assert!(controller.pending_edit_decision_prompt().is_none());

    publish_release.wait();
    assert!(publisher.join().unwrap());
    assert_eq!(
        controller
            .pending_edit_decision_prompt()
            .expect("prompt must remain available after its publication completes")
            .pending_count,
        1
    );
}

#[test]
fn failed_decision_publication_releases_the_fence_for_a_retry() {
    let controller = PlaySessionController::new();
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    controller
        .route_edit(EditOperationTarget::EditWorkspace, lossless("queued"))
        .unwrap();
    controller.request_stop().unwrap();

    assert_eq!(
        controller.with_pending_edit_decision_prompt(|_| Err::<(), _>("center unavailable")),
        Err("center unavailable")
    );
    assert_eq!(
        controller
            .pending_edit_decision_prompt()
            .expect("failed publication must leave the pending prompt retryable")
            .pending_count,
        1
    );
    assert_eq!(
        controller
            .with_pending_edit_decision_prompt(|prompt| {
                assert_eq!(prompt.pending_count, 1);
                Ok::<(), ()>(())
            })
            .unwrap(),
        true
    );
    assert!(!matches!(
        controller.route_edit(EditOperationTarget::EditWorkspace, lossless("after-retry")),
        Err(PlayEditRouteError::PendingResolutionInProgress)
    ));
    assert!(matches!(
        controller.request_play(PlayStartRequest::immediate(PlayKind::Play, None)),
        Err(PlaySessionError::PendingEditDecisionRequired { pending_count: 1 })
    ));
}

#[test]
fn panicking_decision_publication_releases_the_fence_for_a_retry() {
    let controller = PlaySessionController::new();
    controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .unwrap();
    controller
        .route_edit(EditOperationTarget::EditWorkspace, lossless("queued"))
        .unwrap();
    controller.request_stop().unwrap();

    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = controller.with_pending_edit_decision_prompt(|_| -> Result<(), ()> {
            panic!("decision publication panicked")
        });
    }));
    assert!(panic.is_err());
    assert_eq!(
        controller
            .pending_edit_decision_prompt()
            .expect("unwinding must release the publication fence")
            .pending_count,
        1
    );
    assert_eq!(
        controller
            .with_pending_edit_decision_prompt(|prompt| {
                assert_eq!(prompt.pending_count, 1);
                Ok::<(), ()>(())
            })
            .unwrap(),
        true
    );
    assert!(matches!(
        controller.apply_pending_edits(PendingEditApplyBudget::unlimited(), |_| Ok::<(), ()>(())),
        Ok(report) if report.applied.len() == 1
    ));
    assert!(controller
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .is_ok());
}
