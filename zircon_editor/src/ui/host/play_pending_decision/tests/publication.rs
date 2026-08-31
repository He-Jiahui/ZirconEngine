use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use super::super::{
    PlayPendingEditDecisionAdapter, PlayPendingEditDecisionOutcome,
    PLAY_PENDING_EDITS_APPLY_OPTION, PLAY_PENDING_EDITS_DISCARD_OPTION,
};
use super::support::first_pending_selection;
use crate::core::i18n::{EditorI18nService, EditorLocale};
use crate::core::notifications::{DecisionCenterConfig, DecisionNotificationCenter};
use crate::core::play::{PendingEditDecisionPrompt, PendingEditQueueSummary};
use crate::ui::activity::activity_decision_options;

#[test]
fn publishes_one_decision_with_stable_apply_and_discard_rows() {
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())
        .expect("decision center should construct");
    let adapter = PlayPendingEditDecisionAdapter::default();
    let i18n = EditorI18nService::default();
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
    let options = activity_decision_options(&center.pending_snapshot(), &i18n);
    assert_eq!(options.len(), 2);
    assert_eq!(
        options[0]
            .selection_id()
            .selection()
            .unwrap()
            .option_id()
            .as_str(),
        PLAY_PENDING_EDITS_APPLY_OPTION
    );
    assert_eq!(
        options[1]
            .selection_id()
            .selection()
            .unwrap()
            .option_id()
            .as_str(),
        PLAY_PENDING_EDITS_DISCARD_OPTION
    );
    assert_ne!(options[0].selection_id(), options[1].selection_id());
}

#[test]
fn concurrent_prompt_publication_keeps_one_pending_decision() {
    let center = Arc::new(
        DecisionNotificationCenter::new(DecisionCenterConfig::default())
            .expect("decision center should construct"),
    );
    let adapter = Arc::new(PlayPendingEditDecisionAdapter::default());
    let prompt = Arc::new(PendingEditDecisionPrompt::new(PendingEditQueueSummary {
        pending_count: 2,
        payload_bytes: 256,
        oldest_age: Some(Duration::from_secs(4)),
    }));
    let barrier = Arc::new(Barrier::new(2));
    adapter.configure_before_publish_state_lock_hook(Arc::new({
        let barrier = Arc::clone(&barrier);
        move || {
            barrier.wait();
        }
    }));

    let first = {
        let adapter = Arc::clone(&adapter);
        let center = Arc::clone(&center);
        let prompt = Arc::clone(&prompt);
        thread::spawn(move || adapter.publish(&center, &prompt))
    };
    let second = {
        let adapter = Arc::clone(&adapter);
        let center = Arc::clone(&center);
        let prompt = Arc::clone(&prompt);
        thread::spawn(move || adapter.publish(&center, &prompt))
    };

    first
        .join()
        .expect("first publisher should not panic")
        .expect("first publisher should succeed");
    second
        .join()
        .expect("second publisher should not panic")
        .expect("second publisher should reuse the pending decision");
    assert_eq!(center.pending_snapshot().len(), 1);
}

#[test]
fn activity_projection_reprojects_core_decision_when_locale_changes() {
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
        .expect("pending edits should publish a decision");
    let english = activity_decision_options(&center.pending_snapshot(), &i18n);
    let (ticket, option) = first_pending_selection(&center);
    let selection_id = english[0].selection_id().as_str().to_string();
    assert_eq!(english[0].title(), "Unsaved changes");
    assert_eq!(
        english[0].message(),
        "Resolve 1 queued changes (128 bytes; oldest 1s) before starting Play. [Apply changes]"
    );

    i18n.set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap();
    let chinese = activity_decision_options(&center.pending_snapshot(), &i18n);

    assert_eq!(chinese.len(), 2);
    assert_eq!(chinese[0].selection_id().as_str(), selection_id);
    assert_eq!(chinese[0].title(), "未保存的更改");
    assert_eq!(
        chinese[0].message(),
        "开始运行前，请处理 1 项待处理的更改（128 字节；最久 1 秒）。 [应用更改]"
    );
    assert_eq!(center.pending_snapshot()[0].ticket(), &ticket);

    let resolved = center
        .resolve(&ticket, &option)
        .expect("localized presentation must preserve the actionable receipt route");
    assert!(resolved.newly_resolved());
    assert_eq!(resolved.receipt().ticket(), &ticket);
}

#[test]
fn core_receipt_remains_idempotent_without_a_play_selection_map() {
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
    let first = center
        .resolve(&ticket, &option)
        .expect("first selection should resolve");
    let repeated = center
        .resolve(&ticket, &option)
        .expect("repeated selection should return the same receipt");

    assert!(first.newly_resolved());
    assert!(!repeated.newly_resolved());
    assert_eq!(first.receipt(), repeated.receipt());
}

#[test]
fn resolved_receipt_is_not_republished_until_the_adapter_consumes_it() {
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
        .expect("headless receipt should resolve the original Decision");

    adapter
        .publish(&center, &prompt)
        .expect("an unconsumed receipt should retain its Decision ownership");
    assert!(center.pending_snapshot().is_empty());

    adapter
        .consume_resolved_receipts(&center, |_| {
            Ok(PlayPendingEditDecisionOutcome::Discarded { discarded_count: 1 })
        })
        .expect("the resolved receipt should be consumed");
    adapter
        .publish(&center, &prompt)
        .expect("a completed receipt may republish the still-pending prompt");
    assert_eq!(center.pending_snapshot().len(), 1);
}
