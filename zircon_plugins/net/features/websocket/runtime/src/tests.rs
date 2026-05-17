use zircon_plugin_net_runtime::DefaultNetManager;
use zircon_runtime::core::framework::net::{
    NetEndpoint, NetError, NetManager, NetSecurityPolicy, NetWebSocketConnectDescriptor,
    NetWebSocketFrame, NetWebSocketListenerDescriptor,
};

use super::{
    plugin_feature_registration, websocket_runtime_manager, NET_WEBSOCKET_FEATURE_CAPABILITY,
    NET_WEBSOCKET_FEATURE_ID, NET_WEBSOCKET_FEATURE_MANAGER_NAME,
    NET_WEBSOCKET_FEATURE_MODULE_NAME,
};

#[test]
fn websocket_feature_registration_contributes_runtime_module_and_manager() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, NET_WEBSOCKET_FEATURE_ID);
    assert!(report
        .manifest
        .capabilities
        .iter()
        .any(|capability| capability == NET_WEBSOCKET_FEATURE_CAPABILITY));
    let module = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NET_WEBSOCKET_FEATURE_MODULE_NAME)
        .expect("WebSocket feature module should be registered");
    assert_eq!(
        module.managers[0].name.to_string(),
        NET_WEBSOCKET_FEATURE_MANAGER_NAME
    );
}

#[test]
fn websocket_feature_manager_connects_over_real_handshake() {
    let net = websocket_runtime_manager();
    assert!(net.backend_name().contains("+websocket"));
    let listener = net
        .listen_websocket(NetWebSocketListenerDescriptor::new(NetEndpoint::new(
            "127.0.0.1",
            0,
        )))
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

#[test]
fn websocket_feature_manager_enforces_server_path_header_and_subprotocol_policy() {
    let net = websocket_runtime_manager();
    let listener = net
        .listen_websocket(
            NetWebSocketListenerDescriptor::new(NetEndpoint::new("127.0.0.1", 0))
                .with_allowed_path("/socket")
                .with_required_header("x-zircon-net", "enabled")
                .with_allowed_protocol("zircon.rpc"),
        )
        .unwrap();
    let endpoint = net.listener_endpoint(listener).unwrap();

    assert_websocket_policy_rejects(
        &net,
        listener,
        NetWebSocketConnectDescriptor::new(format!(
            "ws://{}:{}/wrong",
            endpoint.host, endpoint.port
        ))
        .with_header("x-zircon-net", "enabled")
        .with_protocol("zircon.rpc"),
    );

    assert_websocket_policy_rejects(
        &net,
        listener,
        NetWebSocketConnectDescriptor::new(format!(
            "ws://{}:{}/socket",
            endpoint.host, endpoint.port
        ))
        .with_protocol("zircon.rpc"),
    );

    assert_websocket_policy_rejects(
        &net,
        listener,
        NetWebSocketConnectDescriptor::new(format!(
            "ws://{}:{}/socket",
            endpoint.host, endpoint.port
        ))
        .with_header("x-zircon-net", "enabled")
        .with_protocol("other.protocol"),
    );

    let connector = net.clone();
    let client_thread = std::thread::spawn(move || {
        connector
            .connect_websocket(
                NetWebSocketConnectDescriptor::new(format!(
                    "ws://{}:{}/socket",
                    endpoint.host, endpoint.port
                ))
                .with_header("x-zircon-net", "enabled")
                .with_protocol("zircon.rpc"),
            )
            .unwrap()
    });
    let server = accept_until_websocket(&net, listener);
    let client = client_thread
        .join()
        .expect("websocket connect thread panicked");

    net.send_websocket_frame(client, NetWebSocketFrame::Text("policy-ok".to_string()))
        .unwrap();
    assert_eq!(
        poll_websocket_until(&net, server),
        NetWebSocketFrame::Text("policy-ok".to_string())
    );
}

#[test]
fn default_type_can_receive_websocket_backend_for_direct_tests() {
    let net: DefaultNetManager = websocket_runtime_manager();

    assert!(net.backend_name().contains("+websocket"));
}

#[test]
fn websocket_feature_manager_rejects_connections_that_violate_security_policy_before_network_io() {
    let net = websocket_runtime_manager();
    let mut tls_required = NetWebSocketConnectDescriptor::new("ws://example.invalid/socket");
    tls_required.security = NetSecurityPolicy::production_tls();

    assert_eq!(
        net.connect_websocket(tls_required).unwrap_err(),
        NetError::SecurityPolicyViolation {
            reason: "WebSocket connection requires WSS by security policy".to_string(),
        }
    );

    let mut pinning_missing = NetWebSocketConnectDescriptor::new("wss://example.invalid/socket");
    pinning_missing.security.certificate_pinning = true;

    assert_eq!(
        net.connect_websocket(pinning_missing).unwrap_err(),
        NetError::SecurityPolicyViolation {
            reason: "WebSocket certificate pinning has no configured pin for host: example.invalid"
                .to_string(),
        }
    );
}

#[test]
fn websocket_feature_manager_accepts_configured_certificate_pin_before_network_io() {
    let net = websocket_runtime_manager();
    let mut descriptor = NetWebSocketConnectDescriptor::new("wss://example.invalid/socket");
    descriptor.security = NetSecurityPolicy::production_tls()
        .with_certificate_pin("example.invalid", "sha256/example");

    let error = net.connect_websocket(descriptor).unwrap_err();
    assert_ne!(
        error,
        NetError::SecurityPolicyViolation {
            reason: "WebSocket certificate pinning has no configured pin for host: example.invalid"
                .to_string(),
        }
    );
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

fn assert_websocket_policy_rejects(
    net: &DefaultNetManager,
    listener: zircon_runtime::core::framework::net::NetListenerId,
    descriptor: NetWebSocketConnectDescriptor,
) {
    let connector = net.clone();
    let client_thread = std::thread::spawn(move || connector.connect_websocket(descriptor));
    for _ in 0..100 {
        assert!(net.accept_websocket(listener, 4).unwrap().is_empty());
        if client_thread.is_finished() {
            let error = client_thread
                .join()
                .expect("websocket rejected connect thread panicked")
                .expect_err("websocket policy should reject connection");
            assert!(
                matches!(error, NetError::Io(ref message) if message.contains("HTTP error: 403")),
                "unexpected websocket policy error: {error:?}"
            );
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("expected WebSocket policy rejection");
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
