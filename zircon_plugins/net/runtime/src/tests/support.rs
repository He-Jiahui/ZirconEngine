use zircon_runtime::core::framework::net::{
    NetConnectionId, NetListenerId, NetManager, NetPacket, NetSocketId,
};

use crate::DefaultNetManager;

pub(super) fn poll_until_packet(net: &DefaultNetManager, socket: NetSocketId) -> Vec<NetPacket> {
    for _ in 0..100 {
        let packets = net.poll_udp(socket, 4).unwrap();
        if !packets.is_empty() {
            return packets;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("expected loopback UDP packet");
}

pub(super) fn accept_until_connection(
    net: &DefaultNetManager,
    listener: NetListenerId,
) -> NetConnectionId {
    for _ in 0..100 {
        let accepted = net.accept_tcp(listener, 4).unwrap();
        if let Some(connection) = accepted.into_iter().next() {
            return connection;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("expected accepted TCP connection");
}

pub(super) fn poll_tcp_until(
    net: &DefaultNetManager,
    connection: NetConnectionId,
    expected_len: usize,
) -> Vec<u8> {
    for _ in 0..100 {
        let payload = net.poll_tcp(connection, expected_len).unwrap();
        if !payload.is_empty() {
            return payload;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("expected TCP payload");
}
