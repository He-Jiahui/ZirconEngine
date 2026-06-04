use zircon_runtime::core::framework::net::{
    ReliableDatagramRecoveryState, ReliableDatagramSimulationProfile,
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
