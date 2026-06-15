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

#[test]
fn oversize_payload_fragments_and_reassembles() {
    let manager = NetReliableUdpRuntimeManager::new(ReliableDatagramConfig {
        mtu_bytes: 4,
        ..ReliableDatagramConfig::default()
    });
    let report = manager.enqueue_reliable_datagram("state", b"abcdefghij".to_vec());

    assert_eq!(report.packets.len(), 3);
    assert_eq!(
        report
            .packets
            .iter()
            .map(|packet| (
                packet.fragment_index,
                packet.fragment_count,
                packet.payload.clone()
            ))
            .collect::<Vec<_>>(),
        vec![
            (0, 3, b"abcd".to_vec()),
            (1, 3, b"efgh".to_vec()),
            (2, 3, b"ij".to_vec()),
        ]
    );

    assert_eq!(
        manager.receive_packet(report.packets[2].clone()).status,
        ReliableDatagramReceiveStatus::AcceptedFragment
    );
    assert_eq!(
        manager.receive_packet(report.packets[0].clone()).status,
        ReliableDatagramReceiveStatus::AcceptedFragment
    );
    let completed = manager.receive_packet(report.packets[1].clone());

    assert_eq!(completed.status, ReliableDatagramReceiveStatus::Reassembled);
    assert_eq!(completed.payload, Some(b"abcdefghij".to_vec()));
}
