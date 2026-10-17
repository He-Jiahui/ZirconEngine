//! 验证提示过期释放容量和身份，存活提示仍拒绝重复及超额；内容键的空值和过长输入保留不同诊断。
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

#[test]
fn toast_lifetime_policy_preserves_subsecond_deadlines_and_inclusive_hour() {
    for (index, lifetime) in [
        Duration::from_nanos(1),
        Duration::from_millis(999),
        Duration::from_secs(3_600),
    ]
    .iter()
    .copied()
    .enumerate()
    {
        let notification = toast(&format!("lifetime_boundary_{index}"), lifetime);
        assert_eq!(notification.lifetime(), lifetime);
        let center = ToastNotificationCenter::new(ToastCenterConfig::new(1).unwrap());
        let now = Duration::from_secs(7);
        let expires_at = now + lifetime;

        center.publish_at(notification, now).unwrap();
        let snapshots = center.snapshot_at(expires_at - Duration::from_nanos(1));
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].notification().lifetime(), lifetime);
        assert_eq!(snapshots[0].expires_at(), expires_at);
        assert!(center.snapshot_at(expires_at).is_empty());
    }
}

#[test]
fn toast_lifetime_policy_rejects_zero_and_over_hour_with_matching_diagnostic() {
    let expected = concat!(
        "toast lifetime must be greater than 0ns and at most 3600s; ",
        "publication time plus lifetime must not overflow"
    );
    for lifetime in [
        Duration::ZERO,
        Duration::from_secs(3_600) + Duration::from_nanos(1),
    ] {
        let error = ToastNotification::new(
            NotificationId::parse("editor.toast.invalid_lifetime").unwrap(),
            NotificationSource::builtin("editor17").unwrap(),
            ToastSeverity::Info,
            "editor.toast.title",
            "editor.toast.message",
            lifetime,
        )
        .unwrap_err();

        assert_eq!(error, ToastNotificationError::InvalidLifetime);
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn toast_expiry_overflow_rejects_without_consuming_identity_or_capacity() {
    let center = ToastNotificationCenter::new(ToastCenterConfig::new(1).unwrap());
    let notification = toast("expiry_overflow", Duration::from_nanos(1));

    assert_eq!(
        center.publish_at(notification.clone(), Duration::MAX),
        Err(ToastNotificationError::InvalidLifetime)
    );
    center.publish_at(notification, Duration::ZERO).unwrap();
    assert_eq!(center.snapshot_at(Duration::ZERO).len(), 1);
}
