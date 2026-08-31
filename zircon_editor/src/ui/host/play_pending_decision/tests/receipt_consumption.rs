use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use super::super::{
    PlayPendingDecisionReceiptDispatchError, PlayPendingEditDecisionAdapter,
    PlayPendingEditDecisionOutcome,
};
use super::support::{deferred_apply_failure, first_pending_selection, publish_foreign_pending};
use crate::core::editing::operation::EditOperationTarget;
use crate::core::editor_message::{EditorMessagePayload, EditorTopic, TransactionMessage};
use crate::core::notifications::{DecisionCenterConfig, DecisionNotificationCenter};
use crate::core::play::{
    PendingEditDecisionPrompt, PendingEditQueueSummary, PlayKind, PlayStartRequest,
};
use crate::ui::host::{EditorHostEventController, EditorManager};
use crate::ui::workbench::state::EditorState;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;

#[test]
fn direct_core_receipt_is_consumed_once_by_the_play_adapter() {
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())
        .expect("decision center should construct");
    let adapter = PlayPendingEditDecisionAdapter::default();
    adapter
        .publish(
            &center,
            &PendingEditDecisionPrompt::new(PendingEditQueueSummary {
                pending_count: 1,
                payload_bytes: 128,
                oldest_age: Some(Duration::from_secs(1)),
            }),
        )
        .expect("pending edits should publish a decision");
    let (ticket, option) = first_pending_selection(&center);
    center
        .resolve(&ticket, &option)
        .expect("headless core resolution should commit a receipt");

    let mut executions = 0_usize;
    let consumed = adapter
        .consume_resolved_receipts(&center, |_| {
            executions += 1;
            Ok(PlayPendingEditDecisionOutcome::Discarded { discarded_count: 1 })
        })
        .expect("the receipt should be consumed through the shared adapter route");
    assert_eq!(consumed.len(), 1);
    assert_eq!(executions, 1);
    assert!(adapter
        .consume_resolved_receipts(&center, |_| {
            executions += 1;
            Ok(PlayPendingEditDecisionOutcome::Discarded { discarded_count: 1 })
        })
        .expect("the committed cursor should make replay a no-op")
        .is_empty());
    assert_eq!(executions, 1);
}

#[test]
fn failed_receipt_execution_retries_once_until_the_cursor_commits() {
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())
        .expect("decision center should construct");
    let adapter = PlayPendingEditDecisionAdapter::default();
    let prompt = PendingEditDecisionPrompt::new(PendingEditQueueSummary {
        pending_count: 1,
        payload_bytes: 128,
        oldest_age: Some(Duration::from_secs(1)),
    });
    adapter
        .publish(&center, &prompt)
        .expect("pending edits should publish a decision");
    let (ticket, option) = first_pending_selection(&center);
    center
        .resolve(&ticket, &option)
        .expect("headless core resolution should commit a receipt");

    let mut first_attempts = 0_usize;
    assert!(adapter
        .consume_resolved_receipts(&center, |_| {
            first_attempts += 1;
            Err(PlayPendingDecisionReceiptDispatchError::UnsupportedOption {
                option: "receipt-execution-failure".to_string(),
            })
        })
        .is_err());
    assert_eq!(first_attempts, 1);

    let mut retry_attempts = 0_usize;
    assert_eq!(
        adapter
            .consume_resolved_receipts(&center, |_| {
                retry_attempts += 1;
                Ok(PlayPendingEditDecisionOutcome::Discarded { discarded_count: 1 })
            })
            .expect("retry should finish the unhandled receipt")
            .len(),
        1
    );
    assert_eq!(retry_attempts, 1);
    assert!(adapter
        .consume_resolved_receipts(&center, |_| {
            retry_attempts += 1;
            Ok(PlayPendingEditDecisionOutcome::Discarded { discarded_count: 1 })
        })
        .expect("the committed cursor should make another retry a no-op")
        .is_empty());
    assert_eq!(retry_attempts, 1);
}

#[test]
fn post_effect_reconcile_failure_does_not_replay_the_committed_receipt() {
    let core = CoreRuntime::new();
    let manager =
        Arc::new(EditorManager::new(&core.handle()).expect("test editor manager should construct"));
    let state = EditorState::with_default_selection_with_context(
        DefaultLevelManager::default().create_default_level(),
        UVec2::new(1280, 720),
        Arc::clone(manager.context()),
    );
    let controller = EditorHostEventController::new(state, manager);
    controller
        .play_sessions()
        .request_play(PlayStartRequest::immediate(PlayKind::Play, None))
        .expect("play should start before the deferred edit is queued");
    controller
        .play_sessions()
        .route_edit(
            EditOperationTarget::EditWorkspace,
            deferred_apply_failure("reconcile-capacity"),
        )
        .expect("lossless edit should queue while play is active");
    controller
        .play_sessions()
        .request_stop()
        .expect("stop should expose the pending-edit prompt");
    controller
        .reconcile_pending_play_decision_from_controller()
        .expect("the controller should publish the pending Decision");
    let context = Arc::clone(controller.context());
    let center = context
        .notifications()
        .decisions()
        .expect("test notification service should expose its Decision center");
    let selection_id = {
        let (ticket, option) = first_pending_selection(center);
        format!("{}:{}", ticket.notification_id(), option)
    };
    let capacity = DecisionCenterConfig::default().pending_capacity();
    let mut relief = None;
    for index in 0..capacity.saturating_sub(1) {
        let pending = publish_foreign_pending(center, &format!("capacity-{index}"));
        relief.get_or_insert(pending);
    }

    let fill_once = Arc::new(AtomicBool::new(true));
    controller
        .play_pending_decisions()
        .configure_before_publish_state_lock_hook(Arc::new({
            let context = Arc::clone(&context);
            let fill_once = Arc::clone(&fill_once);
            move || {
                if fill_once.swap(false, Ordering::AcqRel) {
                    let center = context.notifications().decisions().unwrap();
                    publish_foreign_pending(center, "post-effect-fill");
                }
            }
        }));

    controller
        .resolve_activity_decision(&selection_id)
        .expect("the generic activity route should commit the core Decision receipt");
    let error = controller
        .pump_pending_play_decision_receipts()
        .expect_err("full Decision center should fail only the post-effect reconciliation");
    assert!(matches!(
        error,
        super::super::PlayPendingDecisionReceiptError::Reconcile { .. }
    ));

    let (relief_ticket, relief_option) = relief.expect("foreign capacity fixture should exist");
    center
        .resolve(&relief_ticket, &relief_option)
        .expect("resolving one foreign Decision should free replacement capacity");
    assert_eq!(
        controller
            .pump_pending_play_decision_receipts()
            .expect("the committed receipt should reconcile without replaying its effect"),
        0
    );
    let mut remaining_pending_edits = 0;
    assert!(controller
        .play_sessions()
        .with_pending_edit_decision_prompt(|prompt| {
            remaining_pending_edits = prompt.pending_count;
            Ok::<(), ()>(())
        })
        .unwrap());
    assert_eq!(remaining_pending_edits, 1);
}
