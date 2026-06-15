use crate::{
    NetReliableUdpRuntimeManager, ReliableUdpFragmentHeader, ReliableUdpWireHeader,
    ReliableUdpWirePacket, RELIABLE_UDP_FLAG_FRAGMENT, RELIABLE_UDP_FLAG_LAST_FRAGMENT,
};

#[test]
fn reliable_udp_wire_packet_round_trips_header_ack_bitmap_and_fragment() {
    let packet = ReliableUdpWirePacket::new(
        ReliableUdpWireHeader::new(7, 5, 0b101, 2)
            .with_flags(RELIABLE_UDP_FLAG_FRAGMENT | RELIABLE_UDP_FLAG_LAST_FRAGMENT)
            .with_fragment(ReliableUdpFragmentHeader::new(99, 1, 3)),
        b"payload".to_vec(),
    );

    let bytes = packet.encode();
    assert_eq!(&bytes[0..2], &7_u16.to_le_bytes());
    assert_eq!(&bytes[2..4], &5_u16.to_le_bytes());
    assert_eq!(&bytes[4..8], &0b101_u32.to_le_bytes());
    assert_eq!(bytes[8], 2);
    assert_eq!(
        bytes[9],
        RELIABLE_UDP_FLAG_FRAGMENT | RELIABLE_UDP_FLAG_LAST_FRAGMENT
    );

    let decoded = ReliableUdpWirePacket::decode(&bytes).unwrap();
    assert_eq!(decoded, packet);
    assert_eq!(decoded.header.acked_sequences(), vec![5, 4, 2]);
}

#[test]
fn reliable_udp_wire_ack_matches_pending_window_after_u16_wrap() {
    let manager = NetReliableUdpRuntimeManager::default();
    for _ in 0..=u16::MAX {
        manager.enqueue_reliable_datagram("state", b"payload".to_vec());
    }

    assert_eq!(
        manager.acknowledge_wire_header(ReliableUdpWireHeader::new(0, 0, 0, 0)),
        1
    );
    assert_eq!(
        manager
            .pending_packets()
            .iter()
            .filter(|packet| packet.sequence == u64::from(u16::MAX) + 1)
            .count(),
        0
    );
}
