use zircon_runtime::core::framework::net::{
    NetConnectionState, NetEndpoint, NetManager, NetRuntimeMode,
};

use crate::DefaultNetManager;

use super::support::{accept_until_connection, poll_tcp_until};

#[test]
fn net_runtime_manager_accepts_tcp_client_and_echoes_payloads() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::ListenServer);
    let listener = net.listen_tcp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.listener_endpoint(listener).unwrap();

    let client = net.connect_tcp(&endpoint).unwrap();
    let server = accept_until_connection(&net, listener);

    assert_eq!(
        net.connection_state(client).unwrap(),
        NetConnectionState::Open
    );
    assert_eq!(
        net.connection_state(server).unwrap(),
        NetConnectionState::Open
    );

    assert_eq!(net.send_tcp(client, b"hello").unwrap(), 5);
    assert_eq!(poll_tcp_until(&net, server, 5), b"hello");
    assert_eq!(net.send_tcp(server, b"pong").unwrap(), 4);
    assert_eq!(poll_tcp_until(&net, client, 4), b"pong");

    net.close_connection(client).unwrap();
    net.close_connection(server).unwrap();
}
