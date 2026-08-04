use crate::core::notifications::{DecisionCenterConfig, DecisionNotificationCenter};
use crate::core::play::{PendingEditDecisionPrompt, PendingEditQueueSummary};
use std::time::Duration;

use super::{
    PLAY_PENDING_EDITS_APPLY_OPTION, PLAY_PENDING_EDITS_DISCARD_OPTION,
    PlayPendingEditDecisionAdapter,
};

#[test]
fn publishes_one_decision_with_stable_apply_and_discard_rows() {
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())
        .expect("decision center should construct");
    let adapter = PlayPendingEditDecisionAdapter::default();
    let prompt = PendingEditDecisionPrompt::new(PendingEditQueueSummary {
        pending_count: 2,
        payload_bytes: 256,
        oldest_age: Some(Duration::from_secs(4)),
    });

    adapter
        .publish(&center, &prompt)
        .expect("pending edits should publish a decision");
    adapter
        .publish(&center, &prompt)
        .expect("repeated lifecycle observation should reuse the decision");

    assert_eq!(center.pending_snapshot().len(), 1);
    let options = adapter.pending_options(&center);
    assert_eq!(options.len(), 2);
    assert_eq!(
        options[0].option_id().as_str(),
        PLAY_PENDING_EDITS_APPLY_OPTION
    );
    assert_eq!(
        options[1].option_id().as_str(),
        PLAY_PENDING_EDITS_DISCARD_OPTION
    );
    assert_ne!(options[0].selection_id(), options[1].selection_id());
    assert!(adapter.selection(options[0].selection_id()).is_some());
    assert!(adapter.selection(options[1].selection_id()).is_some());
}

#[test]
fn receipt_selection_remains_addressable_for_idempotent_resolution() {
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
    let option = adapter
        .pending_options(&center)
        .into_iter()
        .next()
        .expect("apply option should be available");
    let first = adapter
        .resolve(&center, option.selection_id())
        .expect("first selection should resolve");
    let repeated = adapter
        .resolve(&center, option.selection_id())
        .expect("repeated selection should return the same receipt");

    assert!(first.newly_resolved());
    assert!(!repeated.newly_resolved());
    assert_eq!(first.receipt(), repeated.receipt());
    assert!(adapter.selection(option.selection_id()).is_some());
}

#[test]
fn resolved_receipt_can_republish_a_pending_decision_after_execution_is_rejected() {
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
    let first_selection = adapter
        .pending_options(&center)
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

    // The controller may reject execution after the receipt commits (for
    // example, because another resolver owns the Play barrier). The pending
    // edit intent must regain a new, actionable Decision instead of remaining
    // blocked behind a resolved receipt.
    adapter
        .publish(&center, &prompt)
        .expect("a rejected execution should republish the pending decision");

    let retry_selection = adapter
        .pending_options(&center)
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
    let resolve = include_str!("resolve.rs");
    let publish = include_str!("publish.rs");

    assert!(!resolve.contains("requeue_pending_play_decision"));
    assert!(resolve.contains("failed to apply queued play edits"));
    assert!(resolve.contains("failed to discard queued play edits"));
    assert!(resolve.contains("reconcile_pending_play_decision(center)?"));
    assert!(publish.contains("with_pending_edit_decision_prompt"));
    assert!(publish.contains("reconcile_pending_play_decision(center)?"));
}

#[test]
fn apply_failure_keeps_the_complete_pending_intent_for_diagnostics() {
    let model = include_str!("model.rs");
    let resolve = include_str!("resolve.rs");
    let notification_toast =
        include_str!("../../retained_host/callback_dispatch/workbench/control.rs");

    assert!(model.contains("intent: PendingEditIntent"));
    assert!(resolve.contains("PlayPendingEditApplyFailure::new(failure.intent, failure.error)"));
    assert!(notification_toast.contains("failure.intent()"));
    assert!(notification_toast.contains("ToastNotification"));
}

#[test]
fn stop_and_backend_exit_publish_through_the_same_pending_decision_adapter() {
    let menu_actions = include_str!("../editor_event_execution/menu_action.rs");
    let host_controller = include_str!("../editor_host_event_controller.rs");

    assert!(menu_actions.contains(
        "controller\n                .publish_pending_edit_decision(transition.pending_edit_prompt.as_ref())"
    ));
    assert!(host_controller.contains(
        "publish_pending_edit_decision(backend_transition.pending_edit_prompt.as_ref())"
    ));
}
