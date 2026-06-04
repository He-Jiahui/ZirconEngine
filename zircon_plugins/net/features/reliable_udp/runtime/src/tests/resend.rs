use zircon_runtime::core::framework::net::{ReliableDatagramConfig, ReliableDatagramRecoveryState};

use crate::NetReliableUdpRuntimeManager;

#[test]
fn reliable_udp_resend_tick_waits_for_timeout_and_tracks_attempts() {
    let manager = NetReliableUdpRuntimeManager::new(ReliableDatagramConfig {
        resend_timeout_ms: 50,
        max_resend_attempts: 3,
        ..ReliableDatagramConfig::default()
    });
    manager.enqueue_reliable_datagram("state", b"payload".to_vec());

    assert!(manager.resend_due(49).is_empty());
    let first_due = manager.resend_due(50);

    assert_eq!(first_due.len(), 1);
    assert_eq!(first_due[0].sequence, 1);
    assert!(manager.resend_due(99).is_empty());
    let second_due = manager.resend_due(100);
    assert_eq!(second_due.len(), 1);
    assert_eq!(manager.stats().resent_packets, 2);
    assert_eq!(
        manager.recovery_state().state,
        ReliableDatagramRecoveryState::Connected
    );
}

#[test]
fn reliable_udp_resend_tick_disconnects_after_resend_attempt_cap() {
    let manager = NetReliableUdpRuntimeManager::new(ReliableDatagramConfig {
        resend_timeout_ms: 25,
        max_resend_attempts: 2,
        ..ReliableDatagramConfig::default()
    });
    manager.enqueue_reliable_datagram("state", b"payload".to_vec());

    assert_eq!(manager.resend_due(25).len(), 1);
    assert_eq!(manager.resend_due(50).len(), 1);
    let capped = manager.resend_due(75);

    assert!(capped.is_empty());
    assert!(manager.pending_packets().is_empty());
    let recovery = manager.recovery_state();
    assert_eq!(recovery.state, ReliableDatagramRecoveryState::Disconnected);
    assert_eq!(recovery.pending_packets, 0);
    assert_eq!(
        recovery.diagnostic.as_deref(),
        Some("reliable datagram resend attempt cap exceeded")
    );
}
