use zircon_runtime::core::framework::net::{
    ReliableDatagramAck, ReliableDatagramConfig, ReliableDatagramSendStatus,
};

use crate::NetReliableUdpRuntimeManager;

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
