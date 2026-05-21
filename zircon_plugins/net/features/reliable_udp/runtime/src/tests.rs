use zircon_runtime::core::framework::net::{
    ReliableDatagramAck, ReliableDatagramConfig, ReliableDatagramReceiveStatus,
    ReliableDatagramRecoveryState, ReliableDatagramSendStatus, ReliableDatagramSimulationProfile,
};

use super::{
    plugin_feature_registration, NetReliableUdpRuntimeManager, NET_RELIABLE_UDP_FEATURE_CAPABILITY,
    NET_RELIABLE_UDP_FEATURE_ID, NET_RELIABLE_UDP_FEATURE_MANAGER_NAME,
    NET_RELIABLE_UDP_FEATURE_MODULE_NAME,
};

#[test]
fn reliable_udp_feature_registration_contributes_runtime_module_and_manager() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, NET_RELIABLE_UDP_FEATURE_ID);
    assert!(report
        .manifest
        .capabilities
        .iter()
        .any(|capability| capability == NET_RELIABLE_UDP_FEATURE_CAPABILITY));
    let module = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NET_RELIABLE_UDP_FEATURE_MODULE_NAME)
        .expect("reliable UDP feature module should be registered");
    assert_eq!(
        module.managers[0].name.to_string(),
        NET_RELIABLE_UDP_FEATURE_MANAGER_NAME
    );
}

#[test]
fn reliable_udp_manager_fragments_tracks_pending_and_acknowledges_sequence() {
    let manager = NetReliableUdpRuntimeManager::new(ReliableDatagramConfig {
        mtu_bytes: 4,
        ..ReliableDatagramConfig::default()
    });

    let report = manager.enqueue_reliable_datagram("state", b"abcdef".to_vec());

    assert_eq!(report.status, ReliableDatagramSendStatus::Fragmented);
    assert_eq!(report.packets.len(), 2);
    assert_eq!(manager.pending_packets().len(), 2);
    assert_eq!(manager.resend_pending(1).len(), 1);
    manager.record_dropped_packet();
    manager.record_rtt_ms(42.0);
    assert_eq!(manager.acknowledge(ReliableDatagramAck::new(1)), 2);
    assert!(manager.pending_packets().is_empty());
    let stats = manager.stats();
    assert_eq!(stats.sent_packets, 2);
    assert_eq!(stats.received_packets, 2);
    assert_eq!(stats.resent_packets, 1);
    assert_eq!(stats.dropped_packets, 1);
    assert_eq!(stats.rtt_ms, 42.0);
}

#[test]
fn reliable_udp_simulation_drops_and_reorders_packets_deterministically() {
    let manager = NetReliableUdpRuntimeManager::default();
    manager.set_simulation_profile(
        ReliableDatagramSimulationProfile::new()
            .with_drop_every_nth_packet(2)
            .with_reorder_window(2),
    );
    let first = manager.enqueue_reliable_datagram("state", b"one".to_vec());
    let second = manager.enqueue_reliable_datagram("state", b"two".to_vec());
    let third = manager.enqueue_reliable_datagram("state", b"three".to_vec());

    let delivery = manager.simulate_outbound_delivery(
        first
            .packets
            .into_iter()
            .chain(second.packets)
            .chain(third.packets),
    );

    assert_eq!(
        delivery
            .delivered_packets
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![3, 1]
    );
    assert_eq!(
        delivery
            .dropped_packets
            .iter()
            .map(|packet| packet.sequence)
            .collect::<Vec<_>>(),
        vec![2]
    );
    assert_eq!(
        delivery.recovery.state,
        ReliableDatagramRecoveryState::Connected
    );
    assert_eq!(manager.stats().dropped_packets, 1);
}

#[test]
fn reliable_udp_receiver_reassembles_out_of_order_fragments_once() {
    let manager = NetReliableUdpRuntimeManager::new(ReliableDatagramConfig {
        mtu_bytes: 3,
        ..ReliableDatagramConfig::default()
    });
    let report = manager.enqueue_reliable_datagram("state", b"abcdefghi".to_vec());
    let packets = report.packets;

    let second = manager.receive_packet(packets[1].clone());
    let first = manager.receive_packet(packets[0].clone());
    let third = manager.receive_packet(packets[2].clone());
    let duplicate = manager.receive_packet(packets[0].clone());

    assert_eq!(
        second.status,
        ReliableDatagramReceiveStatus::AcceptedFragment
    );
    assert_eq!(
        first.status,
        ReliableDatagramReceiveStatus::AcceptedFragment
    );
    assert_eq!(third.status, ReliableDatagramReceiveStatus::Reassembled);
    assert_eq!(third.payload, Some(b"abcdefghi".to_vec()));
    assert_eq!(third.ack, Some(ReliableDatagramAck::new(1)));
    assert_eq!(
        duplicate.status,
        ReliableDatagramReceiveStatus::DuplicateFragment
    );
    assert_eq!(manager.stats().received_packets, 3);
}

#[test]
fn reliable_udp_recovery_state_tracks_drop_threshold_disconnect_and_recovery() {
    let manager = NetReliableUdpRuntimeManager::default();
    manager.set_simulation_profile(
        ReliableDatagramSimulationProfile::new()
            .with_drop_every_nth_packet(1)
            .with_recovery_drop_threshold(2),
    );
    let first = manager.enqueue_reliable_datagram("state", b"one".to_vec());
    let second = manager.enqueue_reliable_datagram("state", b"two".to_vec());

    let delivery =
        manager.simulate_outbound_delivery(first.packets.into_iter().chain(second.packets));

    assert_eq!(delivery.dropped_packets.len(), 2);
    assert_eq!(
        delivery.recovery.state,
        ReliableDatagramRecoveryState::Recovering
    );
    assert_eq!(delivery.recovery.dropped_packets_since_recovery, 2);
    assert_eq!(
        manager.mark_disconnected("ack timeout").state,
        ReliableDatagramRecoveryState::Disconnected
    );
    let recovered = manager.mark_recovered();
    assert_eq!(recovered.state, ReliableDatagramRecoveryState::Connected);
    assert_eq!(recovered.dropped_packets_since_recovery, 0);
}

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
