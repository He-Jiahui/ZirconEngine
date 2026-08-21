use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use super::super::{
    PlayPendingEditDecisionAdapter, PlayPendingEditDecisionOutcome,
    PLAY_PENDING_EDITS_APPLY_OPTION, PLAY_PENDING_EDITS_DISCARD_OPTION,
};
use crate::core::i18n::{EditorI18nService, EditorLocale};
use crate::core::notifications::{DecisionCenterConfig, DecisionNotificationCenter};
use crate::core::play::{PendingEditDecisionPrompt, PendingEditQueueSummary};

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
    let options = adapter.pending_options(&center, &i18n);
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
fn pending_options_reproject_core_decision_when_locale_changes() {
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
    let english = adapter.pending_options(&center, &i18n);
    let ticket = english[0].selection().ticket().clone();
    let selection_id = english[0].selection_id().to_string();
    assert_eq!(english[0].title(), "Unsaved changes");
    assert_eq!(
        english[0].message(),
        "Resolve 1 queued changes (128 bytes; oldest 1s) before starting Play. [Apply changes]"
    );

    i18n.set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap();
    let chinese = adapter.pending_options(&center, &i18n);

    assert_eq!(chinese.len(), 2);
    assert_eq!(chinese[0].selection_id(), selection_id);
    assert_eq!(chinese[0].selection().ticket(), &ticket);
    assert_eq!(chinese[0].title(), "未保存的更改");
    assert_eq!(
        chinese[0].message(),
        "开始运行前，请处理 1 项待处理的更改（128 字节；最久 1 秒）。 [应用更改]"
    );
    assert_eq!(center.pending_snapshot()[0].ticket(), &ticket);

    let resolved = adapter
        .resolve(&center, &selection_id)
        .expect("localized presentation must preserve the actionable receipt route");
    assert!(resolved.newly_resolved());
    assert_eq!(resolved.receipt().ticket(), &ticket);
}

#[test]
fn receipt_selection_remains_addressable_for_idempotent_resolution() {
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::default())
        .expect("decision center should construct");
    let adapter = PlayPendingEditDecisionAdapter::default();
    let i18n = EditorI18nService::default();

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
        .pending_options(&center, &i18n)
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
fn resolved_receipt_is_not_republished_until_the_adapter_consumes_it() {
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
    let selection = adapter
        .pending_options(&center, &i18n)
        .into_iter()
        .next()
        .expect("apply option should be available")
        .selection();
    center
        .resolve(selection.ticket(), selection.option_id())
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
