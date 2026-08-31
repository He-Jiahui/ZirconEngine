use std::time::Duration;

use super::super::{PlayPendingEditDecisionAdapter, PlayPendingEditDecisionOutcome};
use super::support::first_pending_selection;
use crate::core::notifications::{DecisionCenterConfig, DecisionNotificationCenter};
use crate::core::play::{PendingEditDecisionPrompt, PendingEditQueueSummary};

#[test]
fn consumed_receipt_can_republish_a_still_pending_decision() {
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
        .expect("initial pending edits should publish a decision");
    let first_selection = first_pending_selection(&center);
    assert!(center
        .resolve(&first_selection.0, &first_selection.1)
        .expect("receipt should resolve")
        .newly_resolved());

    adapter
        .consume_resolved_receipts(&center, |_| {
            Ok(PlayPendingEditDecisionOutcome::Discarded { discarded_count: 0 })
        })
        .expect("the resolved receipt should commit through the adapter");

    // A completed receipt releases the previous Decision ticket. If the
    // controller still reports pending edits, the prompt must regain a new,
    // actionable Decision rather than remaining blocked behind the old one.
    adapter
        .publish(&center, &prompt)
        .expect("a completed receipt should republish the pending decision");

    let retry_selection = first_pending_selection(&center);
    assert_ne!(retry_selection.0, first_selection.0);
    assert_eq!(center.pending_snapshot().len(), 1);
}

#[test]
fn receipt_consumer_reconciles_the_current_prompt_after_effect_execution() {
    let resolve = include_str!("../resolve.rs");
    let publish = include_str!("../publish.rs");

    assert!(!resolve.contains("requeue_pending_play_decision"));
    assert!(resolve.contains("PlayPendingDecisionReceiptDispatchError::from)?;"));
    assert!(resolve.contains("PlayPendingDecisionReceiptDispatchError::UnsupportedOption"));
    assert!(!resolve.contains("failed to apply queued play edits"));
    assert!(!resolve.contains("failed to discard queued play edits"));
    let consume = resolve
        .find("consume_resolved_receipts(center")
        .expect("controller should commit receipts through the adapter");
    let reconcile = resolve
        .find("self.reconcile_pending_play_decision(center)")
        .expect("controller should reconcile the latest pending prompt");
    assert!(consume < reconcile);
    let execute = resolve
        .split("fn execute_pending_play_decision_selection")
        .nth(1)
        .expect("controller should keep the effect executor explicit");
    assert!(!execute.contains("reconcile_pending_play_decision"));
    assert!(publish.contains("with_pending_edit_decision_prompt"));
    assert!(publish.contains("reconcile_pending_play_decision(center)?"));
}

#[test]
fn apply_failure_keeps_the_complete_pending_intent_for_the_receipt_notification() {
    let model = include_str!("../model.rs");
    let resolve = include_str!("../resolve.rs");

    assert!(model.contains("intent: PendingEditIntent"));
    assert!(resolve.contains("PlayPendingEditApplyFailure::new(failure.intent, failure.error)"));
    assert!(resolve.contains("failure.intent()"));
    assert!(resolve.contains("ToastNotification"));
    assert!(resolve.contains("publish_pending_play_decision_outcome_toasts"));
}

#[test]
fn stop_and_backend_exit_publish_through_the_same_pending_decision_adapter() {
    let menu_actions = include_str!("../../editor_event_execution/menu_action.rs");
    let host_controller = include_str!("../../editor_host_event_controller.rs");

    assert!(menu_actions.contains("reconcile_pending_play_decision_from_controller()?"));
    assert!(host_controller.contains("reconcile_pending_play_decision_from_controller()"));
}
