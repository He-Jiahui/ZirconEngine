use zircon_runtime::core::framework::net::{
    NetConnectionState, NetEndpoint, NetEvent, NetManager, NetRuntimeMode,
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
    let events = net.drain_events(16);
    assert!(events.iter().any(|event| matches!(
        event,
        NetEvent::ConnectionAccepted {
            listener: accepted_listener,
            connection,
            transport,
            ..
        } if *accepted_listener == listener && *connection == server && transport.is_tcp()
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        NetEvent::ConnectionStateChanged {
            connection,
            transport,
            state: NetConnectionState::Open,
        } if *connection == client && transport.is_tcp()
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        NetEvent::ConnectionStateChanged {
            connection,
            transport,
            state: NetConnectionState::Open,
        } if *connection == server && transport.is_tcp()
    )));

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
    let events = net.drain_events(16);
    assert!(events.iter().any(|event| matches!(
        event,
        NetEvent::ConnectionClosed {
            connection,
            transport,
        } if *connection == client && transport.is_tcp()
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        NetEvent::ConnectionStateChanged {
            connection,
            transport,
            state: NetConnectionState::Closed,
        } if *connection == client && transport.is_tcp()
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        NetEvent::ConnectionClosed {
            connection,
            transport,
        } if *connection == server && transport.is_tcp()
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        NetEvent::ConnectionStateChanged {
            connection,
            transport,
            state: NetConnectionState::Closed,
        } if *connection == server && transport.is_tcp()
    )));
}

use zircon_runtime::core::framework::net::{NetConnectionId, NetError};

#[test]
fn net_tcp_read_error_keeps_failed_event_and_query_in_sync() {
    use std::time::{Duration, Instant};
    use tokio::{net::TcpSocket, runtime::Builder};

    let net = DefaultNetManager::for_mode(NetRuntimeMode::ListenServer);
    let listener = net.listen_tcp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let address = net
        .listener_endpoint(listener)
        .unwrap()
        .to_socket_addr()
        .unwrap();

    let peer_runtime = Builder::new_current_thread().enable_io().build().unwrap();
    let socket = TcpSocket::new_v4().unwrap();
    socket.set_zero_linger().unwrap();
    let peer = peer_runtime.block_on(socket.connect(address)).unwrap();
    let server = accept_until_connection(&net, listener);
    assert_eq!(
        net.connection_state(server).unwrap(),
        NetConnectionState::Open
    );
    let _ = net.drain_events(16);
    let inbound_before = net.diagnostics().inbound_bytes;
    assert_eq!(inbound_before, 0);

    // Zero linger makes the peer close abortive (RST) rather than graceful (EOF).
    drop(peer);
    let deadline = Instant::now() + Duration::from_secs(2);
    let io_error = loop {
        let result = net.poll_tcp(server, 1);
        // Do not mistake a worker reply timeout for a socket error.
        assert!(
            Instant::now() < deadline,
            "no TCP read error before deadline for {server:?}; last poll: {result:?}"
        );
        match result {
            Err(NetError::Io(error)) => break error,
            Ok(payload) if payload.is_empty() => std::thread::sleep(Duration::from_millis(1)),
            result => {
                panic!("expected an idle read or TCP I/O error for {server:?}, got {result:?}")
            }
        }
    };

    let events = net.drain_events(16);
    let failed_count = events
        .iter()
        .filter(|event| {
            matches!(
                event,
                NetEvent::ConnectionStateChanged {
                    connection,
                    transport,
                    state: NetConnectionState::Failed,
                } if *connection == server && transport.is_tcp()
            )
        })
        .count();
    assert_eq!(
        failed_count, 1,
        "TCP poll returned Io({io_error:?}); expected one Failed event for {server:?}: {events:?}"
    );
    assert_eq!(
        net.connection_state(server).unwrap(),
        NetConnectionState::Failed,
        "TCP poll returned Io({io_error:?}) and emitted Failed for {server:?}"
    );
    assert_eq!(net.diagnostics().inbound_bytes, inbound_before);
}

#[test]
fn net_tcp_read_error_idle_and_zero_budget_reads_keep_connection_open() {
    let (net, client, server) = open_idle_tcp_pair();

    assert!(net.poll_tcp(server, 1).unwrap().is_empty());
    assert_open_without_tcp_state_event(&net, server);

    assert_eq!(net.send_tcp(client, b"x").unwrap(), 1);
    assert!(net.poll_tcp(server, 0).unwrap().is_empty());
    assert_open_without_tcp_state_event(&net, server);
    assert_eq!(poll_tcp_until(&net, server, 1), b"x");
}

#[test]
fn net_tcp_read_error_unknown_connection_does_not_change_state() {
    let (net, _, server) = open_idle_tcp_pair();
    let unknown = NetConnectionId::new(u64::MAX);

    assert_eq!(
        net.poll_tcp(unknown, 1),
        Err(NetError::UnknownConnection {
            connection: unknown
        })
    );
    assert_eq!(
        net.connection_state(unknown),
        Err(NetError::UnknownConnection {
            connection: unknown
        })
    );
    assert_open_without_tcp_state_event(&net, server);
}

#[test]
fn net_tcp_read_error_worker_shutdown_does_not_guess_failed_state() {
    let (net, _, server) = open_idle_tcp_pair();
    net.shutdown_worker_for_tests();
    let before = net.connection_state(server).unwrap();
    let _ = net.drain_events(16);

    let result = net.poll_tcp(server, 1);
    assert!(
        matches!(&result, Err(NetError::Io(_))),
        "expected outer I/O error with worker stopped, got {result:?}"
    );
    assert_eq!(
        net.connection_state(server).unwrap(),
        before,
        "poll without a worker read reply changed state for {server:?}: {result:?}"
    );
    let events = net.drain_events(16);
    assert!(
        !events.iter().any(|event| matches!(
            event,
            NetEvent::ConnectionStateChanged {
                state: NetConnectionState::Failed,
                ..
            }
        )),
        "poll without a worker read reply emitted Failed: {events:?}"
    );
}

fn open_idle_tcp_pair() -> (DefaultNetManager, NetConnectionId, NetConnectionId) {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::ListenServer);
    let listener = net.listen_tcp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.listener_endpoint(listener).unwrap();
    let client = net.connect_tcp(&endpoint).unwrap();
    let server = accept_until_connection(&net, listener);
    assert_eq!(
        net.connection_state(server).unwrap(),
        NetConnectionState::Open
    );
    let _ = net.drain_events(16);
    (net, client, server)
}

fn assert_open_without_tcp_state_event(net: &DefaultNetManager, connection: NetConnectionId) {
    assert_eq!(
        net.connection_state(connection).unwrap(),
        NetConnectionState::Open
    );
    let events = net.drain_events(16);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, NetEvent::ConnectionStateChanged { .. })),
        "unexpected connection state change after TCP poll: {events:?}"
    );
}
