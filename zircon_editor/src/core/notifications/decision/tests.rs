use super::*;
use std::sync::{Arc, Barrier};
use std::thread;

fn center() -> DecisionNotificationCenter {
    DecisionNotificationCenter::new(DecisionCenterConfig::default()).unwrap()
}

#[test]
fn core_root_mounts_the_typed_notification_contract() {
    let core_root = include_str!("../../mod.rs");

    assert!(core_root
        .lines()
        .any(|line| line.trim() == "pub mod notifications;"));
}

#[test]
fn editor_context_owns_an_empty_decision_notification_center() {
    use crate::core::context::EditorContextBuilder;
    use crate::core::jobs::test_job_scheduler;

    let context = EditorContextBuilder::new(test_job_scheduler()).build();

    assert!(context
        .notifications()
        .decisions()
        .unwrap()
        .pending_snapshot()
        .is_empty());
}

fn configured_center(
    pending_capacity: usize,
    receipt_capacity: usize,
) -> DecisionNotificationCenter {
    DecisionNotificationCenter::new(
        DecisionCenterConfig::new(pending_capacity, receipt_capacity).unwrap(),
    )
    .unwrap()
}

fn option(id: &str) -> DecisionOption {
    DecisionOption::new(
        DecisionOptionId::parse(id).unwrap(),
        format!("editor.decision.{id}"),
    )
    .unwrap()
}

fn notification(suffix: &str, cancel: bool) -> DecisionNotification {
    let apply = DecisionOptionId::parse("apply").unwrap();
    let discard = DecisionOptionId::parse("discard").unwrap();
    let notification = DecisionNotification::new(
        NotificationId::parse(format!("editor.play.{suffix}")).unwrap(),
        NotificationSource::builtin("editor04").unwrap(),
        "editor.play.pending.title",
        "editor.play.pending.message",
        vec![option("apply"), option("discard")],
    )
    .unwrap()
    .with_default_option(apply)
    .unwrap();
    if cancel {
        notification.with_cancel_option(discard).unwrap()
    } else {
        notification
    }
}

#[test]
fn publish_and_resolve_receipt_once() {
    let center = center();
    let notification = notification("pending_edits", true);
    let ticket = center.publish(notification).unwrap();

    let report = center
        .resolve(&ticket, &DecisionOptionId::parse("apply").unwrap())
        .unwrap();

    assert!(report.newly_resolved());
    assert_eq!(report.receipt().sequence().value(), 1);
    assert!(center.pending_snapshot().is_empty());
    assert_eq!(
        center
            .receipts_since(center.initial_cursor())
            .unwrap()
            .receipts(),
        &[report.receipt().clone()]
    );
}

#[test]
fn repeated_same_receipt_is_idempotent() {
    let center = center();
    let notification = notification("idempotent", true);
    let apply = DecisionOptionId::parse("apply").unwrap();
    let ticket = center.publish(notification).unwrap();

    let first = center.resolve(&ticket, &apply).unwrap();
    let repeated = center.resolve(&ticket, &apply).unwrap();

    assert!(!repeated.newly_resolved());
    assert_eq!(repeated.receipt(), first.receipt());
    assert_eq!(
        center
            .receipts_since(center.initial_cursor())
            .unwrap()
            .receipts()
            .len(),
        1
    );
}

#[test]
fn conflicting_second_receipt_is_rejected() {
    let center = center();
    let notification = notification("conflict", true);
    let ticket = center.publish(notification).unwrap();
    center
        .resolve(&ticket, &DecisionOptionId::parse("apply").unwrap())
        .unwrap();

    assert!(matches!(
        center.resolve(&ticket, &DecisionOptionId::parse("discard").unwrap()),
        Err(DecisionNotificationError::AlreadyResolved { .. })
    ));
}

#[test]
fn cancel_requires_declared_option() {
    let center = center();
    let without_cancel = notification("without_cancel", false);
    let without_cancel_ticket = center.publish(without_cancel).unwrap();
    assert!(matches!(
        center.cancel(&without_cancel_ticket),
        Err(DecisionNotificationError::CancellationNotAllowed { .. })
    ));
    assert_eq!(center.pending_snapshot().len(), 1);

    let with_cancel = notification("with_cancel", true);
    let with_cancel_ticket = center.publish(with_cancel).unwrap();
    let report = center.cancel(&with_cancel_ticket).unwrap();
    assert_eq!(report.receipt().option_id().as_str(), "discard");
}

#[test]
fn repeated_cancel_receipt_is_idempotent() {
    let center = center();
    let notification = notification("cancel_idempotent", true);
    let ticket = center.publish(notification).unwrap();

    let first = center.cancel(&ticket).unwrap();
    let repeated = center.cancel(&ticket).unwrap();

    assert!(first.newly_resolved());
    assert!(!repeated.newly_resolved());
    assert_eq!(repeated.receipt(), first.receipt());
}

#[test]
fn stale_ticket_cannot_resolve_reused_notification_id() {
    let center = configured_center(1, 1);
    let apply = DecisionOptionId::parse("apply").unwrap();
    let old_ticket = center.publish(notification("reused", true)).unwrap();
    center.resolve(&old_ticket, &apply).unwrap();

    let eviction_ticket = center.publish(notification("eviction", true)).unwrap();
    center.resolve(&eviction_ticket, &apply).unwrap();
    let current_ticket = center.publish(notification("reused", true)).unwrap();

    assert!(matches!(
        center.resolve(&old_ticket, &apply),
        Err(DecisionNotificationError::StaleTicket { .. })
    ));
    let pending = center.pending_snapshot();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].ticket(), &current_ticket);
}

#[test]
fn foreign_ticket_and_cursor_are_rejected() {
    let first = center();
    let second = center();
    let foreign_ticket = first.publish(notification("foreign", true)).unwrap();
    second.publish(notification("foreign", true)).unwrap();

    assert!(matches!(
        second.resolve(&foreign_ticket, &DecisionOptionId::parse("apply").unwrap()),
        Err(DecisionNotificationError::ForeignTicket { .. })
    ));
    assert!(matches!(
        second.receipts_since(first.initial_cursor()),
        Err(DecisionNotificationError::ForeignCursor { .. })
    ));
    assert_eq!(second.pending_snapshot().len(), 1);
}

#[test]
fn oversized_payload_is_rejected() {
    let too_long_notification = format!("editor.play.{}", "a".repeat(MAX_NOTIFICATION_ID_BYTES));
    assert!(matches!(
        NotificationId::parse(too_long_notification),
        Err(DecisionNotificationError::FieldTooLong { .. })
    ));
    assert!(matches!(
        DecisionOptionId::parse("a".repeat(MAX_DECISION_OPTION_ID_BYTES + 1)),
        Err(DecisionNotificationError::FieldTooLong { .. })
    ));
    assert!(matches!(
        NotificationSource::plugin("a".repeat(MAX_NOTIFICATION_SOURCE_ID_BYTES + 1)),
        Err(DecisionNotificationError::FieldTooLong { .. })
    ));
    assert!(matches!(
        DecisionOption::new(
            DecisionOptionId::parse("apply").unwrap(),
            "a".repeat(MAX_LOCALIZATION_KEY_BYTES + 1),
        ),
        Err(DecisionNotificationError::FieldTooLong { .. })
    ));

    let options = (0..=MAX_DECISION_OPTIONS)
        .map(|index| option(&format!("option_{index}")))
        .collect();
    assert!(matches!(
        DecisionNotification::new(
            NotificationId::parse("editor.play.too_many").unwrap(),
            NotificationSource::builtin("editor17").unwrap(),
            "editor.play.title",
            "editor.play.message",
            options,
        ),
        Err(DecisionNotificationError::TooManyOptions { .. })
    ));
    assert!(matches!(
        DecisionNotification::new(
            NotificationId::parse("editor.play.long_title").unwrap(),
            NotificationSource::builtin("editor17").unwrap(),
            "a".repeat(MAX_LOCALIZATION_KEY_BYTES + 1),
            "editor.play.message",
            vec![option("apply"), option("discard")],
        ),
        Err(DecisionNotificationError::FieldTooLong { .. })
    ));
}

#[test]
fn pending_capacity_rejects_without_mutation() {
    let center = configured_center(1, 4);
    center.publish(notification("first", true)).unwrap();

    assert_eq!(
        center.publish(notification("second", true)),
        Err(DecisionNotificationError::PendingCapacityReached { capacity: 1 })
    );
    assert_eq!(center.pending_snapshot().len(), 1);
    assert_eq!(
        center.pending_snapshot()[0].notification().id().as_str(),
        "editor.play.first"
    );
}

#[test]
fn resolving_a_notification_releases_pending_capacity() {
    let center = configured_center(1, 4);
    let first = center
        .publish(notification("first_capacity", true))
        .unwrap();

    center
        .resolve(&first, &DecisionOptionId::parse("apply").unwrap())
        .unwrap();
    let second = center
        .publish(notification("second_capacity", true))
        .unwrap();

    assert_eq!(center.pending_snapshot().len(), 1);
    assert_eq!(
        second.notification_id().as_str(),
        "editor.play.second_capacity"
    );
}

#[test]
fn expired_cursor_recovers_oldest_retained_receipt() {
    let center = configured_center(1, 2);
    let initial_cursor = center.initial_cursor();
    let apply = DecisionOptionId::parse("apply").unwrap();
    for suffix in ["first", "second", "third"] {
        let ticket = center.publish(notification(suffix, true)).unwrap();
        center.resolve(&ticket, &apply).unwrap();
    }

    let resume_cursor = match center.receipts_since(initial_cursor) {
        Err(DecisionNotificationError::CursorExpired { resume_cursor, .. }) => resume_cursor,
        other => panic!("expected cursor gap, got {other:?}"),
    };
    let recovered = center.receipts_since(resume_cursor).unwrap();
    assert_eq!(recovered.receipts().len(), 2);
    assert_eq!(recovered.receipts()[0].sequence().value(), 2);
    assert_eq!(recovered.receipts()[1].sequence().value(), 3);
}

#[test]
fn concurrent_same_option_resolves_once() {
    let center = Arc::new(center());
    let ticket = center.publish(notification("same_race", true)).unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let handles = (0..2)
        .map(|_| {
            let center = Arc::clone(&center);
            let ticket = ticket.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                center.resolve(&ticket, &DecisionOptionId::parse("apply").unwrap())
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let reports = handles
        .into_iter()
        .map(|handle| handle.join().unwrap().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        reports
            .iter()
            .filter(|report| report.newly_resolved())
            .count(),
        1
    );
    assert_eq!(reports[0].receipt(), reports[1].receipt());
}

#[test]
fn concurrent_conflicting_options_have_one_winner() {
    let center = Arc::new(center());
    let ticket = center.publish(notification("conflict_race", true)).unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let handles = ["apply", "discard"]
        .into_iter()
        .map(|option_id| {
            let center = Arc::clone(&center);
            let ticket = ticket.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                center.resolve(&ticket, &DecisionOptionId::parse(option_id).unwrap())
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(
                result,
                Err(DecisionNotificationError::AlreadyResolved { .. })
            ))
            .count(),
        1
    );
}

#[test]
fn concurrent_publish_honors_pending_capacity() {
    let center = Arc::new(configured_center(1, 4));
    let barrier = Arc::new(Barrier::new(3));
    let handles = ["first_race", "second_race"]
        .into_iter()
        .map(|suffix| {
            let center = Arc::clone(&center);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                center.publish(notification(suffix, true))
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(
                result,
                Err(DecisionNotificationError::PendingCapacityReached { .. })
            ))
            .count(),
        1
    );
    assert_eq!(center.pending_snapshot().len(), 1);
}

#[test]
fn cancel_and_resolve_are_linearized() {
    let center = Arc::new(center());
    let ticket = center.publish(notification("cancel_race", true)).unwrap();
    let barrier = Arc::new(Barrier::new(3));
    let cancel = {
        let center = Arc::clone(&center);
        let ticket = ticket.clone();
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            barrier.wait();
            center.cancel(&ticket)
        })
    };
    let resolve = {
        let center = Arc::clone(&center);
        let ticket = ticket.clone();
        let barrier = Arc::clone(&barrier);
        thread::spawn(move || {
            barrier.wait();
            center.resolve(&ticket, &DecisionOptionId::parse("apply").unwrap())
        })
    };
    barrier.wait();
    let results = [cancel.join().unwrap(), resolve.join().unwrap()];

    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(
                result,
                Err(DecisionNotificationError::AlreadyResolved { .. })
            ))
            .count(),
        1
    );
}

#[test]
fn bounded_receipt_history_reports_cursor_gap() {
    let center = configured_center(1, 2);
    let initial_cursor = center.initial_cursor();
    let apply = DecisionOptionId::parse("apply").unwrap();
    let mut sequences = Vec::new();
    for suffix in ["first", "second", "third"] {
        let notification = notification(suffix, true);
        let ticket = center.publish(notification).unwrap();
        sequences.push(
            center
                .resolve(&ticket, &apply)
                .unwrap()
                .receipt()
                .sequence(),
        );
    }

    assert_eq!(
        center.receipts_since(initial_cursor),
        Err(DecisionNotificationError::CursorExpired {
            requested: 0,
            oldest_available: sequences[1],
            resume_cursor: DecisionReceiptCursor::before(center.instance_id(), sequences[1]),
        })
    );
    let batch = center
        .receipts_since(DecisionReceiptCursor::before(
            center.instance_id(),
            sequences[1],
        ))
        .unwrap();
    assert_eq!(batch.receipts().len(), 2);
    assert_eq!(batch.next_cursor().value(), sequences[2].value());
}
