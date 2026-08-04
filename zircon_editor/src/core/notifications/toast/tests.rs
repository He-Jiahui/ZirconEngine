use std::time::Duration;

use crate::core::notifications::{NotificationId, NotificationSource};

use super::{
    ToastCenterConfig, ToastNotification, ToastNotificationCenter, ToastNotificationError,
    ToastSeverity,
};

fn toast(suffix: &str, lifetime: Duration) -> ToastNotification {
    ToastNotification::new(
        NotificationId::parse(format!("editor.toast.{suffix}")).unwrap(),
        NotificationSource::builtin("editor17").unwrap(),
        ToastSeverity::Info,
        "editor.toast.title",
        "editor.toast.message",
        lifetime,
    )
    .unwrap()
}

#[test]
fn expiration_releases_toast_capacity() {
    let center = ToastNotificationCenter::new(ToastCenterConfig::new(1).unwrap());
    center
        .publish_at(toast("first", Duration::from_secs(1)), Duration::ZERO)
        .unwrap();
    center
        .publish_at(
            toast("second", Duration::from_secs(1)),
            Duration::from_secs(1),
        )
        .unwrap();
    let snapshots = center.snapshot_at(Duration::from_secs(1));
    assert_eq!(snapshots.len(), 1);
    assert_eq!(
        snapshots[0].notification().id().as_str(),
        "editor.toast.second"
    );
}

#[test]
fn live_toasts_reject_duplicate_identity_and_overflow() {
    let center = ToastNotificationCenter::new(ToastCenterConfig::new(1).unwrap());
    let first = toast("first", Duration::from_secs(2));
    center.publish_at(first.clone(), Duration::ZERO).unwrap();
    assert!(matches!(
        center.publish_at(first, Duration::ZERO),
        Err(ToastNotificationError::DuplicateNotification { .. })
    ));
    assert!(matches!(
        center.publish_at(toast("second", Duration::from_secs(2)), Duration::ZERO),
        Err(ToastNotificationError::CapacityReached { capacity: 1 })
    ));
}

#[test]
fn toast_content_keys_distinguish_empty_and_oversized_input() {
    let id = NotificationId::parse("editor.toast.invalid_content").unwrap();
    let source = NotificationSource::builtin("editor17").unwrap();
    assert!(matches!(
        ToastNotification::new(
            id.clone(),
            source.clone(),
            ToastSeverity::Info,
            "",
            "editor.toast.message",
            Duration::from_secs(1),
        ),
        Err(ToastNotificationError::EmptyField { field: "title key" })
    ));
    assert!(matches!(
        ToastNotification::new(
            id,
            source,
            ToastSeverity::Info,
            "editor.toast.title",
            "a".repeat(257),
            Duration::from_secs(1),
        ),
        Err(ToastNotificationError::FieldTooLong {
            field: "message key",
            maximum: 256,
            actual: 257,
        })
    ));
}
