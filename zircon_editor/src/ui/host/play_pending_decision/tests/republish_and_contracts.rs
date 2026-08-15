use std::time::Duration;

use super::super::{PlayPendingEditDecisionAdapter, PlayPendingEditDecisionOutcome};
use crate::core::i18n::EditorI18nService;
use crate::core::notifications::{DecisionCenterConfig, DecisionNotificationCenter};
use crate::core::play::{PendingEditDecisionPrompt, PendingEditQueueSummary};

#[test]
fn consumed_receipt_can_republish_a_still_pending_decision() {
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())
        .expect("decision center should construct");
    let adapter = PlayPendingEditDecisionAdapter::default();
    let i18n = EditorI18nService::default();
    let prompt = PendingEditDecisionPrompt::new(PendingEditQueueSummary {
        pending_count: 1,
        payload_bytes: 128,
        oldest_age: Some(Duration::from_secs(1)),
    });

    adapter
        .publish(&center, &prompt)
        .expect("initial pending edits should publish a decision");
    let first_selection = adapter
        .pending_options(&center, &i18n)
        .into_iter()
        .next()
        .expect("initial apply option should be available")
        .selection_id()
        .to_string();
    assert!(
        adapter
            .resolve(&center, &first_selection)
            .expect("receipt should resolve")
            .newly_resolved()
    );

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

    let retry_selection = adapter
        .pending_options(&center, &i18n)
        .into_iter()
        .next()
        .expect("republished apply option should be available")
        .selection_id()
        .to_string();
    assert_ne!(retry_selection, first_selection);
    assert_eq!(center.pending_snapshot().len(), 1);
}

#[test]
fn options_reconcile_the_current_prompt_after_receipt_execution_cannot_begin() {
    let resolve = include_str!("../resolve.rs");
    let publish = include_str!("../publish.rs");

    assert!(!resolve.contains("requeue_pending_play_decision"));
    assert!(resolve.contains("failed to apply queued play edits"));
    assert!(resolve.contains("failed to discard queued play edits"));
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
fn apply_failure_keeps_the_complete_pending_intent_for_diagnostics() {
    let model = include_str!("../model.rs");
    let resolve = include_str!("../resolve.rs");
    let notification_toast =
        include_str!("../../../retained_host/callback_dispatch/workbench/control.rs");

    assert!(model.contains("intent: PendingEditIntent"));
    assert!(resolve.contains("PlayPendingEditApplyFailure::new(failure.intent, failure.error)"));
    assert!(notification_toast.contains("failure.intent()"));
    assert!(notification_toast.contains("ToastNotification"));
}

#[test]
fn stop_and_backend_exit_publish_through_the_same_pending_decision_adapter() {
    let menu_actions = include_str!("../../editor_event_execution/menu_action.rs");
    let host_controller = include_str!("../../editor_host_event_controller.rs");

    assert!(menu_actions.contains(
        "controller\n                .publish_pending_edit_decision(transition.pending_edit_prompt.as_ref())"
    ));
    assert!(host_controller.contains(
        "publish_pending_edit_decision(backend_transition.pending_edit_prompt.as_ref())"
    ));
}
