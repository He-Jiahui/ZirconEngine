use zircon_plugin_net_runtime::DefaultNetManager;
use zircon_runtime::core::framework::net::{
    NetConnectionId, NetError, NetListenerId, NetManager, NetWebSocketConnectDescriptor,
    NetWebSocketFrame,
};

pub(super) fn accept_until_websocket(
    net: &DefaultNetManager,
    listener: NetListenerId,
) -> NetConnectionId {
    for _ in 0..100 {
        let accepted = net.accept_websocket(listener, 4).unwrap();
        if let Some(connection) = accepted.into_iter().next() {
            return connection;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    panic!("expected accepted WebSocket connection");
}

pub(super) fn assert_websocket_policy_rejects(
    net: &DefaultNetManager,
    listener: NetListenerId,
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

pub(super) fn poll_websocket_until(
    net: &DefaultNetManager,
    connection: NetConnectionId,
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
