use zircon_runtime::core::framework::net::{
    NetConnectionState, NetEndpoint, NetEvent, NetHttpMethod, NetHttpRequestDescriptor,
    NetHttpResponseDescriptor, NetHttpRouteDescriptor, NetManager, NetRequestId, NetRuntimeMode,
    NetWebSocketCloseReason, NetWebSocketConnectDescriptor, NetWebSocketFrame, RpcDescriptor,
    RpcDirection,
};
use zircon_runtime::{plugin::RuntimePlugin, plugin::RuntimePluginRegistrationReport};

use super::{runtime_plugin, DefaultNetManager, NET_MODULE_NAME};

#[test]
fn net_plugin_registration_contributes_runtime_module() {
    let report = RuntimePluginRegistrationReport::from_plugin(&runtime_plugin());

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(report
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == NET_MODULE_NAME));
    assert_eq!(
        report.package_manifest.modules[0].target_modes,
        vec![
            zircon_runtime::RuntimeTargetMode::ServerRuntime,
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
        ]
    );
}

#[test]
fn net_plugin_manifest_advertises_layered_optional_features() {
    let manifest = runtime_plugin().package_manifest();

    let feature_ids = manifest
        .optional_features
        .iter()
        .map(|feature| feature.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        feature_ids,
        vec![
            "net.http",
            "net.websocket",
            "net.rpc",
            "net.replication",
            "net.reliable_udp",
            "net.content_download",
        ]
    );
    assert!(manifest
        .options
        .iter()
        .any(|option| option.key == "net.runtime_mode"));
    assert!(manifest
        .event_catalogs
        .iter()
        .any(|catalog| catalog.namespace == "net.runtime_events"));
}

#[test]
fn default_net_manager_sends_udp_packet_to_bound_socket() {
    let net = DefaultNetManager::default();
    let socket = net.bind_udp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.local_endpoint(socket).unwrap();

    assert_eq!(net.send_udp(socket, &endpoint, b"ping").unwrap(), 4);
    let packets = poll_until_packet(&net, socket);

    assert_eq!(packets[0].payload, b"ping");
    net.close_socket(socket).unwrap();
}

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

#[test]
fn net_runtime_manager_reports_mode_diagnostics_and_events() {
    let net = DefaultNetManager::for_mode(NetRuntimeMode::DedicatedServer);
    let listener = net.listen_tcp(&NetEndpoint::new("127.0.0.1", 0)).unwrap();

    let diagnostics = net.diagnostics();
    assert_eq!(diagnostics.mode, NetRuntimeMode::DedicatedServer);
    assert_eq!(diagnostics.open_tcp_listeners, 1);
    assert_eq!(diagnostics.open_tcp_connections, 0);

    let events = net.drain_events(8);
    assert!(events.iter().any(|event| matches!(
        event,
        NetEvent::ListenerStarted {
            listener: started,
            transport,
            ..
        } if *started == listener && transport.is_tcp()
    )));
}

#[test]
fn rpc_descriptor_records_direction_schema_and_quota() {
    let descriptor = RpcDescriptor::new("chat.send_message", RpcDirection::ClientToServer)
        .with_payload_schema("schema://net/chat/send-message.v1")
        .with_max_calls_per_second(24)
        .with_max_payload_bytes(2048);

    assert_eq!(descriptor.id, "chat.send_message");
    assert_eq!(descriptor.direction, RpcDirection::ClientToServer);
    assert_eq!(
        descriptor.payload_schema.as_deref(),
        Some("schema://net/chat/send-message.v1")
    );
    assert_eq!(descriptor.max_calls_per_second, Some(24));
    assert_eq!(descriptor.max_payload_bytes, Some(2048));
}

#[test]
fn net_runtime_dispatches_registered_http_route() {
    let net = DefaultNetManager::default();
    let route = net
        .register_http_route(
            NetHttpRouteDescriptor::new("/health", [NetHttpMethod::Get]),
            NetHttpResponseDescriptor::new(NetRequestId::new(0), 200, b"ok".to_vec())
                .with_header("content-type", "text/plain"),
        )
        .unwrap();

    let response = net
        .send_http_request(NetHttpRequestDescriptor::new(
            NetRequestId::new(7),
            NetHttpMethod::Get,
            "http://127.0.0.1/health",
        ))
        .unwrap();

    assert_eq!(response.request, NetRequestId::new(7));
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, b"ok");
    assert_eq!(response.body_bytes, 2);
    assert_eq!(net.diagnostics().open_http_routes, 1);
    net.unregister_http_route(route).unwrap();
    assert_eq!(net.diagnostics().open_http_routes, 0);
}

#[test]
fn net_runtime_serves_registered_http_route_over_real_socket() {
    let net = DefaultNetManager::default();
    net.register_http_route(
        NetHttpRouteDescriptor::new("/socket-health", [NetHttpMethod::Get]),
        NetHttpResponseDescriptor::new(NetRequestId::new(0), 200, b"socket-ok".to_vec())
            .with_header("content-type", "text/plain"),
    )
    .unwrap();
    let listener = net.listen_http(&NetEndpoint::new("127.0.0.1", 0)).unwrap();
    let endpoint = net.listener_endpoint(listener).unwrap();

    let response = net
        .send_http_request(NetHttpRequestDescriptor::new(
            NetRequestId::new(17),
            NetHttpMethod::Get,
            format!("http://{}:{}/socket-health", endpoint.host, endpoint.port),
        ))
        .unwrap();

    assert_eq!(response.request, NetRequestId::new(17));
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, b"socket-ok");
}

#[test]
fn net_runtime_queues_websocket_frames_with_budget() {
    let net = DefaultNetManager::default();
    let (client, server) = net.open_websocket_loopback().unwrap();

    net.send_websocket_frame(client, NetWebSocketFrame::Text("hello".to_string()))
        .unwrap();
    net.send_websocket_frame(client, NetWebSocketFrame::Binary(vec![1, 2, 3]))
        .unwrap();

    assert_eq!(
        net.poll_websocket_frames(server, 1).unwrap(),
        vec![NetWebSocketFrame::Text("hello".to_string())]
    );
    assert_eq!(
        net.poll_websocket_frames(server, 8).unwrap(),
        vec![NetWebSocketFrame::Binary(vec![1, 2, 3])]
    );

    net.send_websocket_frame(
        server,
        NetWebSocketFrame::Close(NetWebSocketCloseReason::normal("done")),
    )
    .unwrap();
    assert!(matches!(
        net.poll_websocket_frames(client, 8).unwrap().as_slice(),
        [NetWebSocketFrame::Close(reason)] if reason.reason == "done"
    ));
    assert_eq!(
        net.connection_state(client).unwrap(),
        NetConnectionState::Closed
    );
}

#[test]
fn net_runtime_connects_websocket_over_real_handshake() {
    let net = DefaultNetManager::default();
    let listener = net
        .listen_websocket(&NetEndpoint::new("127.0.0.1", 0))
        .unwrap();
    let endpoint = net.listener_endpoint(listener).unwrap();
    let connector = net.clone();
    let client_thread = std::thread::spawn(move || {
        connector
            .connect_websocket(NetWebSocketConnectDescriptor::new(format!(
                "ws://{}:{}/socket",
                endpoint.host, endpoint.port
            )))
            .unwrap()
    });
    let server = accept_until_websocket(&net, listener);
    let client = client_thread
        .join()
        .expect("websocket connect thread panicked");

    net.send_websocket_frame(client, NetWebSocketFrame::Text("hello-real".to_string()))
        .unwrap();
    assert_eq!(
        poll_websocket_until(&net, server),
        NetWebSocketFrame::Text("hello-real".to_string())
    );

    net.send_websocket_frame(server, NetWebSocketFrame::Text("echo-real".to_string()))
        .unwrap();
    assert_eq!(
        poll_websocket_until(&net, client),
        NetWebSocketFrame::Text("echo-real".to_string())
    );
}

fn poll_until_packet(
    net: &DefaultNetManager,
    socket: zircon_runtime::core::framework::net::NetSocketId,
) -> Vec<zircon_runtime::core::framework::net::NetPacket> {
    for _ in 0..100 {
        let packets = net.poll_udp(socket, 4).unwrap();
        if !packets.is_empty() {
            return packets;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("expected loopback UDP packet");
}

fn accept_until_connection(
    net: &DefaultNetManager,
    listener: zircon_runtime::core::framework::net::NetListenerId,
) -> zircon_runtime::core::framework::net::NetConnectionId {
    for _ in 0..100 {
        let accepted = net.accept_tcp(listener, 4).unwrap();
        if let Some(connection) = accepted.into_iter().next() {
            return connection;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("expected accepted TCP connection");
}

fn poll_tcp_until(
    net: &DefaultNetManager,
    connection: zircon_runtime::core::framework::net::NetConnectionId,
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

fn accept_until_websocket(
    net: &DefaultNetManager,
    listener: zircon_runtime::core::framework::net::NetListenerId,
) -> zircon_runtime::core::framework::net::NetConnectionId {
    for _ in 0..100 {
        let accepted = net.accept_websocket(listener, 4).unwrap();
        if let Some(connection) = accepted.into_iter().next() {
            return connection;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("expected accepted WebSocket connection");
}

fn poll_websocket_until(
    net: &DefaultNetManager,
    connection: zircon_runtime::core::framework::net::NetConnectionId,
) -> NetWebSocketFrame {
    for _ in 0..100 {
        let frames = net.poll_websocket_frames(connection, 4).unwrap();
        if let Some(frame) = frames.into_iter().next() {
            return frame;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("expected WebSocket frame");
}
