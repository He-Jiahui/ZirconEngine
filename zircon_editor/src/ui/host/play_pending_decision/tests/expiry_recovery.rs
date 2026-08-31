use std::time::Duration;

use super::super::adapter::{
    ExpiredReceiptRecovery, ExpiredReceiptRepublish, PlayPendingReceiptConsumeError,
};
use super::super::{
    PlayPendingDecisionReceiptRecoveryError, PlayPendingEditDecisionAdapter,
    PlayPendingEditDecisionOutcome,
};
use super::support::{first_pending_selection, publish_foreign_receipt};
use crate::core::notifications::{DecisionCenterConfig, DecisionNotificationCenter};
use crate::core::play::{PendingEditDecisionPrompt, PendingEditQueueSummary};

#[test]
fn expired_receipt_range_requires_a_new_explicit_play_choice() {
    let center = DecisionNotificationCenter::new(
        DecisionCenterConfig::new(8, 2).expect("small test capacities should construct"),
    )
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
    let stale_selection = first_pending_selection(&center);
    center
        .resolve(&stale_selection.0, &stale_selection.1)
        .expect("headless core resolution should commit the retained Play receipt");
    publish_foreign_receipt(&center, "first");
    publish_foreign_receipt(&center, "second");

    let mut executions = 0_usize;
    let resume_cursor = match adapter.consume_resolved_receipts(&center, |_| {
        executions += 1;
        Ok(PlayPendingEditDecisionOutcome::Discarded { discarded_count: 1 })
    }) {
        Err(PlayPendingReceiptConsumeError::CursorExpired { resume_cursor }) => resume_cursor,
        other => panic!("expected an explicit receipt-expiry recovery, received {other:?}"),
    };
    assert_eq!(
        executions, 0,
        "an evicted choice must not execute by guesswork"
    );
    let recovery_error = adapter
        .recover_expired_receipts(&center, resume_cursor, |_| {
            Ok(ExpiredReceiptRepublish::ExistingDecision)
        })
        .expect_err("a missing replacement publication must reject recovery");
    assert!(matches!(
        recovery_error,
        PlayPendingDecisionReceiptRecoveryError::ReplacementDecisionNotEstablished
    ));
    assert!(matches!(
        adapter.consume_resolved_receipts(&center, |_| {
            Ok(PlayPendingEditDecisionOutcome::Discarded { discarded_count: 1 })
        }),
        Err(PlayPendingReceiptConsumeError::CursorExpired { .. })
    ));
    publish_foreign_receipt(&center, "after-failed-recovery");

    adapter
        .recover_expired_receipts(&center, resume_cursor, |stale_cutoff| {
            if !adapter.publish_replacement_after_expiry(&center, &prompt, stale_cutoff)? {
                return Ok(ExpiredReceiptRepublish::ExistingDecision);
            }
            let replacement = first_pending_selection(&center);
            center.resolve(&replacement.0, &replacement.1)?;
            Ok(ExpiredReceiptRepublish::Published)
        })
        .expect("recovery should install only the pre-publication stale cutoff");
    assert_eq!(
        adapter
            .consume_resolved_receipts(&center, |_| {
                executions += 1;
                Ok(PlayPendingEditDecisionOutcome::Discarded { discarded_count: 1 })
            })
            .expect("replacement receipt must remain after the stale cutoff")
            .len(),
        1
    );
    assert_eq!(executions, 1);

    publish_foreign_receipt(&center, "after-replacement-first");
    publish_foreign_receipt(&center, "after-replacement-second");
    publish_foreign_receipt(&center, "after-replacement-third");
    let resume_cursor = match adapter.consume_resolved_receipts(&center, |_| {
        panic!("completed replacement state must not dispatch again")
    }) {
        Err(PlayPendingReceiptConsumeError::CursorExpired { resume_cursor }) => resume_cursor,
        other => panic!("expected a later foreign-only expiry, received {other:?}"),
    };
    assert!(matches!(
        adapter
            .recover_expired_receipts(&center, resume_cursor, |_| {
                panic!("the superseded lost ticket must not require another replacement")
            })
            .unwrap(),
        ExpiredReceiptRecovery::CursorAdvanced { .. }
    ));
}

#[test]
fn expired_owned_receipt_without_a_pending_prompt_keeps_the_cursor_expired() {
    let center = DecisionNotificationCenter::new(
        DecisionCenterConfig::new(8, 2).expect("small test capacities should construct"),
    )
    .expect("decision center should construct");
    let adapter = PlayPendingEditDecisionAdapter::default();
    let prompt = PendingEditDecisionPrompt::new(PendingEditQueueSummary {
        pending_count: 1,
        payload_bytes: 128,
        oldest_age: Some(Duration::from_secs(1)),
    });
    adapter.publish(&center, &prompt).unwrap();
    let stale_selection = first_pending_selection(&center);
    center
        .resolve(&stale_selection.0, &stale_selection.1)
        .unwrap();
    publish_foreign_receipt(&center, "evict-owned-first");
    publish_foreign_receipt(&center, "evict-owned-second");

    let resume_cursor = match adapter.consume_resolved_receipts(&center, |_| {
        panic!("an evicted owned receipt must not execute by guesswork")
    }) {
        Err(PlayPendingReceiptConsumeError::CursorExpired { resume_cursor }) => resume_cursor,
        other => panic!("expected cursor expiry, received {other:?}"),
    };
    let error = adapter
        .recover_expired_receipts(&center, resume_cursor, |_| {
            Ok(ExpiredReceiptRepublish::NotRequired)
        })
        .expect_err("a lost owned choice still requires an explicit replacement");

    assert!(matches!(
        error,
        PlayPendingDecisionReceiptRecoveryError::ReplacementPromptUnavailable
    ));
    assert!(matches!(
        adapter.consume_resolved_receipts(&center, |_| {
            panic!("the evicted choice must remain unexecuted")
        }),
        Err(PlayPendingReceiptConsumeError::CursorExpired { .. })
    ));
}

#[test]
fn expired_receipt_recovery_retains_publish_errors() {
    let center = DecisionNotificationCenter::new(
        DecisionCenterConfig::new(8, 2).expect("small test capacities should construct"),
    )
    .expect("decision center should construct");
    let adapter = PlayPendingEditDecisionAdapter::default();
    let prompt = PendingEditDecisionPrompt::new(PendingEditQueueSummary {
        pending_count: 1,
        payload_bytes: 128,
        oldest_age: Some(Duration::from_secs(1)),
    });
    adapter.publish(&center, &prompt).unwrap();
    let stale_selection = first_pending_selection(&center);
    center
        .resolve(&stale_selection.0, &stale_selection.1)
        .unwrap();
    publish_foreign_receipt(&center, "evict-publish-error-first");
    publish_foreign_receipt(&center, "evict-publish-error-second");

    let resume_cursor = match adapter.consume_resolved_receipts(&center, |_| {
        panic!("an evicted owned receipt must not execute by guesswork")
    }) {
        Err(PlayPendingReceiptConsumeError::CursorExpired { resume_cursor }) => resume_cursor,
        other => panic!("expected cursor expiry, received {other:?}"),
    };
    let error = adapter
        .recover_expired_receipts(&center, resume_cursor, |_| {
            Err(super::super::PlayPendingDecisionPublishError::SequenceExhausted)
        })
        .expect_err("a replacement publication failure must remain typed");

    assert!(matches!(
        error,
        PlayPendingDecisionReceiptRecoveryError::Publish(
            super::super::PlayPendingDecisionPublishError::SequenceExhausted
        )
    ));
}

#[test]
fn foreign_only_receipt_expiry_advances_without_a_play_prompt() {
    let center = DecisionNotificationCenter::new(
        DecisionCenterConfig::new(8, 2).expect("small test capacities should construct"),
    )
    .expect("decision center should construct");
    let adapter = PlayPendingEditDecisionAdapter::default();
    publish_foreign_receipt(&center, "foreign-only-first");
    publish_foreign_receipt(&center, "foreign-only-second");
    publish_foreign_receipt(&center, "foreign-only-third");

    let resume_cursor = match adapter.consume_resolved_receipts(&center, |_| {
        panic!("foreign receipts must not dispatch Play effects")
    }) {
        Err(PlayPendingReceiptConsumeError::CursorExpired { resume_cursor }) => resume_cursor,
        other => panic!("expected cursor expiry, received {other:?}"),
    };
    let recovery = adapter
        .recover_expired_receipts(&center, resume_cursor, |_| {
            panic!("foreign-only expiry must not request a Play replacement")
        })
        .unwrap();

    assert_eq!(
        recovery,
        ExpiredReceiptRecovery::CursorAdvanced {
            owned_receipt_after_cutoff: false,
        }
    );
    assert!(adapter
        .consume_resolved_receipts(&center, |_| {
            panic!("foreign receipts must remain ignored")
        })
        .unwrap()
        .is_empty());
}

#[test]
fn live_play_receipt_after_frozen_cutoff_remains_consumable() {
    let center = DecisionNotificationCenter::new(
        DecisionCenterConfig::new(8, 2).expect("small test capacities should construct"),
    )
    .expect("decision center should construct");
    let adapter = PlayPendingEditDecisionAdapter::default();
    let prompt = PendingEditDecisionPrompt::new(PendingEditQueueSummary {
        pending_count: 1,
        payload_bytes: 128,
        oldest_age: Some(Duration::from_secs(1)),
    });
    publish_foreign_receipt(&center, "live-cutoff-first");
    publish_foreign_receipt(&center, "live-cutoff-second");
    publish_foreign_receipt(&center, "live-cutoff-third");
    adapter.publish(&center, &prompt).unwrap();
    let selection = first_pending_selection(&center);

    let resume_cursor = match adapter.consume_resolved_receipts(&center, |_| {
        panic!("the live Decision has not been resolved yet")
    }) {
        Err(PlayPendingReceiptConsumeError::CursorExpired { resume_cursor }) => resume_cursor,
        other => panic!("expected cursor expiry, received {other:?}"),
    };
    let recovery = adapter
        .recover_expired_receipts(&center, resume_cursor, |_| {
            panic!("an unresolved owned Decision must remain the explicit choice")
        })
        .unwrap();
    assert_eq!(
        recovery,
        ExpiredReceiptRecovery::CursorAdvanced {
            owned_receipt_after_cutoff: false,
        }
    );

    center.resolve(&selection.0, &selection.1).unwrap();
    let resume_cursor = match adapter.consume_resolved_receipts(&center, |_| {
        panic!("the newly resolved receipt must remain retained across cursor recovery")
    }) {
        Err(PlayPendingReceiptConsumeError::CursorExpired { resume_cursor }) => resume_cursor,
        other => panic!("expected the advanced foreign window to expire once, received {other:?}"),
    };
    let recovery = adapter
        .recover_expired_receipts(&center, resume_cursor, |_| {
            panic!("a retained post-cutoff Play receipt must not be replaced")
        })
        .unwrap();
    assert_eq!(
        recovery,
        ExpiredReceiptRecovery::CursorAdvanced {
            owned_receipt_after_cutoff: true,
        }
    );
    let mut executions = 0;
    let outcomes = adapter
        .consume_resolved_receipts(&center, |_| {
            executions += 1;
            Ok(PlayPendingEditDecisionOutcome::Discarded { discarded_count: 1 })
        })
        .unwrap();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(executions, 1);
}
