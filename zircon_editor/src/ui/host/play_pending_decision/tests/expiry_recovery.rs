use std::time::Duration;

use super::super::adapter::{
    ExpiredReceiptRecovery, ExpiredReceiptRepublish, PlayPendingReceiptConsumeError,
};
use super::super::{PlayPendingEditDecisionAdapter, PlayPendingEditDecisionOutcome};
use super::support::publish_foreign_receipt;
use crate::core::i18n::EditorI18nService;
use crate::core::notifications::{DecisionCenterConfig, DecisionNotificationCenter};
use crate::core::play::{PendingEditDecisionPrompt, PendingEditQueueSummary};

#[test]
fn expired_receipt_range_requires_a_new_explicit_play_choice() {
    let center = DecisionNotificationCenter::new(
        DecisionCenterConfig::new(8, 2).expect("small test capacities should construct"),
    )
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
        .expect("pending edits should publish a decision");
    let stale_selection = adapter
        .pending_options(&center, &i18n)
        .into_iter()
        .next()
        .expect("apply option should be available")
        .selection();
    center
        .resolve(stale_selection.ticket(), stale_selection.option_id())
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
    assert!(recovery_error.contains("did not establish a replacement Decision"));
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
            let replacement = adapter
                .pending_options(&center, &i18n)
                .into_iter()
                .next()
                .expect("the replacement Decision must expose a new explicit choice")
                .selection();
            center
                .resolve(replacement.ticket(), replacement.option_id())
                .map_err(|error| error.to_string())?;
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
    let i18n = EditorI18nService::default();
    let prompt = PendingEditDecisionPrompt::new(PendingEditQueueSummary {
        pending_count: 1,
        payload_bytes: 128,
        oldest_age: Some(Duration::from_secs(1)),
    });
    adapter.publish(&center, &prompt).unwrap();
    let stale_selection = adapter
        .pending_options(&center, &i18n)
        .into_iter()
        .next()
        .unwrap()
        .selection();
    center
        .resolve(stale_selection.ticket(), stale_selection.option_id())
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

    assert!(error.contains("owned Apply/Discard choice was lost"));
    assert!(matches!(
        adapter.consume_resolved_receipts(&center, |_| {
            panic!("the evicted choice must remain unexecuted")
        }),
        Err(PlayPendingReceiptConsumeError::CursorExpired { .. })
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
    let i18n = EditorI18nService::default();
    let prompt = PendingEditDecisionPrompt::new(PendingEditQueueSummary {
        pending_count: 1,
        payload_bytes: 128,
        oldest_age: Some(Duration::from_secs(1)),
    });
    publish_foreign_receipt(&center, "live-cutoff-first");
    publish_foreign_receipt(&center, "live-cutoff-second");
    publish_foreign_receipt(&center, "live-cutoff-third");
    adapter.publish(&center, &prompt).unwrap();
    let selection = adapter
        .pending_options(&center, &i18n)
        .into_iter()
        .next()
        .unwrap()
        .selection();

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

    center
        .resolve(selection.ticket(), selection.option_id())
        .unwrap();
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
