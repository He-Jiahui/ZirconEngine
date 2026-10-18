use super::*;
use crate::core::notifications::{NotificationSource, ToastSeverity};

fn notification(id: &str, lifetime: Duration) -> ToastNotification {
    ToastNotification::new(
        NotificationId::parse(id).unwrap(),
        NotificationSource::builtin("editor.delivery.test").unwrap(),
        ToastSeverity::Error,
        "editor.notification.command_failed.title",
        "The operation failed.",
        lifetime,
    )
    .unwrap()
}
fn queue(capacity: usize) -> ToastDeliveryQueue {
    ToastDeliveryQueue::new(ToastCenterConfig::new(capacity).unwrap())
}

#[test]
fn activity_toast_delivery_waits_in_publish_order_until_each_exact_first_ack() {
    let queue = queue(3);
    let lifetime = Duration::from_secs(3);
    queue
        .publish_at(
            notification("editor.event.9.failed", lifetime),
            Duration::ZERO,
        )
        .unwrap();
    queue
        .publish_at(
            notification("editor.event.10.failed", lifetime),
            Duration::ZERO,
        )
        .unwrap();
    let shown = Duration::from_secs(100);
    let pending = queue.snapshot_at(shown);
    assert_eq!(pending.len(), 2);
    assert_eq!(
        pending[0].notification().id().as_str(),
        "editor.event.9.failed"
    );
    assert!(pending.iter().all(|entry| entry.expires_at().is_none()));
    assert_eq!(pending[1].remaining_lifetime_at(shown), lifetime);
    assert!(!queue
        .acknowledge_at(
            pending[1].notification().id(),
            pending[1].delivery_token(),
            shown
        )
        .unwrap());
    assert!(queue
        .acknowledge_at(
            pending[0].notification().id(),
            pending[0].delivery_token(),
            shown
        )
        .unwrap());
    let deadline = shown + lifetime;
    assert_eq!(
        queue.snapshot_at(deadline - Duration::from_nanos(1)).len(),
        2
    );
    let second = queue.snapshot_at(deadline);
    assert_eq!(second.len(), 1);
    assert_eq!(
        second[0].notification().id(),
        pending[1].notification().id()
    );
    assert_eq!(second[0].expires_at(), None);
    assert!(queue
        .acknowledge_at(
            second[0].notification().id(),
            second[0].delivery_token(),
            Duration::from_secs(200)
        )
        .unwrap());
    assert_eq!(
        queue
            .snapshot_at(Duration::from_secs(200) + lifetime - Duration::from_nanos(1))
            .len(),
        1
    );
    assert!(queue
        .snapshot_at(Duration::from_secs(200) + lifetime)
        .is_empty());
}

#[test]
fn activity_toast_delivery_rejects_duplicate_late_and_replaced_identity_acks() {
    let queue = queue(2);
    let id = NotificationId::parse("editor.delivery.same").unwrap();
    let lifetime = Duration::from_secs(1);
    queue
        .publish_at(notification(id.as_str(), lifetime), Duration::ZERO)
        .unwrap();
    let old = queue.snapshot_at(Duration::ZERO)[0].clone();
    assert!(queue
        .acknowledge_at(&id, old.delivery_token(), Duration::from_secs(10))
        .unwrap());
    assert!(!queue
        .acknowledge_at(
            &id,
            old.delivery_token(),
            Duration::from_secs(10) + lifetime / 2
        )
        .unwrap());
    assert!(
        queue.snapshot_at(Duration::from_secs(11)).is_empty(),
        "duplicate ACK must not extend expiry"
    );
    queue
        .publish_at(notification(id.as_str(), lifetime), Duration::from_secs(12))
        .unwrap();
    let replacement = queue.snapshot_at(Duration::from_secs(12))[0].clone();
    assert_ne!(old.delivery_token(), replacement.delivery_token());
    assert!(!queue
        .acknowledge_at(&id, old.delivery_token(), Duration::from_secs(20))
        .unwrap());
    assert!(!queue.dismiss(&id, old.delivery_token()));
    assert!(queue.dismiss(&id, replacement.delivery_token()));
    assert!(!queue
        .acknowledge_at(&id, replacement.delivery_token(), Duration::from_secs(20))
        .unwrap());
    assert!(queue.snapshot_at(Duration::from_secs(20)).is_empty());
}

#[test]
fn activity_toast_delivery_bound_and_clock_overflow_keep_the_pending_instance() {
    let queue = queue(1);
    let id = NotificationId::parse("editor.delivery.overflow").unwrap();
    queue
        .publish_at(
            notification(id.as_str(), Duration::from_nanos(1)),
            Duration::MAX,
        )
        .unwrap();
    let pending = queue.snapshot_at(Duration::MAX)[0].clone();
    assert!(matches!(
        queue.publish_at(
            notification("editor.delivery.extra", Duration::from_secs(1)),
            Duration::MAX
        ),
        Err(ToastNotificationError::CapacityReached { capacity: 1 })
    ));
    assert_eq!(
        queue.acknowledge_at(&id, pending.delivery_token(), Duration::MAX),
        Err(ToastNotificationError::PresentationClockOverflow)
    );
    assert_eq!(queue.snapshot_at(Duration::MAX)[0], pending);
    assert!(queue
        .acknowledge_at(&id, pending.delivery_token(), Duration::ZERO)
        .unwrap());
    assert!(queue.snapshot_at(Duration::from_nanos(1)).is_empty());
}

#[test]
fn activity_toast_delivery_sequence_exhaustion_never_reuses_a_ticket() {
    let queue = queue(1);
    queue.state.lock().unwrap().next_token = Some(u64::MAX);
    let id = NotificationId::parse("editor.delivery.last").unwrap();
    queue
        .publish_at(
            notification(id.as_str(), Duration::from_secs(1)),
            Duration::ZERO,
        )
        .unwrap();
    let last = queue.snapshot_at(Duration::ZERO)[0].clone();
    assert_eq!(last.delivery_token(), u64::MAX);
    assert!(queue.dismiss(&id, last.delivery_token()));
    assert_eq!(
        queue.publish_at(
            notification(id.as_str(), Duration::from_secs(1)),
            Duration::ZERO
        ),
        Err(ToastNotificationError::PresentationSequenceExhausted)
    );
}
