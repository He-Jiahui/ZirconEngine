use zircon_runtime::core::framework::net::{
    ReliableDatagramRecoveryState, ReliableDatagramSimulationProfile,
};

use crate::NetReliableUdpRuntimeManager;

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
