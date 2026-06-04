use zircon_runtime::core::framework::net::{
    ReliableDatagramAck, ReliableDatagramConfig, ReliableDatagramReceiveStatus,
};

use crate::NetReliableUdpRuntimeManager;

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
