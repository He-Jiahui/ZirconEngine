use zircon_runtime::core::framework::net::{
    ReliableDatagramConfig, ReliableDatagramRecoveryState, ReliableDatagramSimulationProfile,
};

use crate::NetReliableUdpRuntimeManager;

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
fn thirty_percent_loss_delivers_in_order() {
    let sender = NetReliableUdpRuntimeManager::new(ReliableDatagramConfig {
        resend_timeout_ms: 10,
        max_resend_attempts: 4,
        ..ReliableDatagramConfig::default()
    });
    let receiver = NetReliableUdpRuntimeManager::default();
    sender.set_simulation_profile(
        ReliableDatagramSimulationProfile::new().with_drop_every_nth_packet(3),
    );

    let packets = (1_u8..=10)
        .flat_map(|value| {
            sender
                .enqueue_reliable_datagram("state", vec![value])
                .packets
        })
        .collect::<Vec<_>>();

    let first_delivery = sender.simulate_outbound_delivery(packets);
    let mut delivered = Vec::new();
    for packet in first_delivery.delivered_packets {
        for payload in receiver.receive_ordered_packet(packet) {
            delivered.push(payload[0]);
        }
    }
    assert_eq!(delivered, vec![1, 2]);

    let resend = sender.resend_due(10);
    for packet in resend {
        for payload in receiver.receive_ordered_packet(packet) {
            delivered.push(payload[0]);
        }
    }

    assert_eq!(delivered, (1_u8..=10).collect::<Vec<_>>());
    assert_eq!(receiver.pending_ordered_payload_count(), 0);
}
