use zircon_runtime::core::framework::net::{NetEndpoint, NetEvent, NetManager};

use crate::DefaultNetManager;

use super::support::poll_until_packet;

#[test]
fn default_net_manager_sends_udp_packet_to_bound_socket() {
    let net = DefaultNetManager::default();
    let socket = net.bind_udp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.local_endpoint(socket).unwrap();

    assert_eq!(net.send_udp(socket, &endpoint, b"ping").unwrap(), 4);
    let packets = poll_until_packet(&net, socket);

    assert_eq!(packets[0].payload, b"ping");
    net.close_socket(socket).unwrap();

    assert!(net.drain_events(8).iter().any(
        |event| matches!(event, NetEvent::UdpSocketClosed { socket: closed } if *closed == socket)
    ));
}
