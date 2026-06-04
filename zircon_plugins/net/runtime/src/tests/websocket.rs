use zircon_runtime::core::framework::net::{
    NetConnectionState, NetEndpoint, NetError, NetManager, NetWebSocketCloseReason,
    NetWebSocketConnectDescriptor, NetWebSocketFrame, NetWebSocketListenerDescriptor,
};

use crate::DefaultNetManager;

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
fn base_net_runtime_requires_websocket_feature_for_real_handshake() {
    let net = DefaultNetManager::default();

    assert_eq!(
        net.listen_websocket(NetWebSocketListenerDescriptor::new(NetEndpoint::new(
            "127.0.0.1",
            0,
        )))
        .unwrap_err(),
        NetError::ProtocolUnavailable {
            capability: "runtime.feature.net.websocket".to_string(),
        }
    );
    assert_eq!(
        net.connect_websocket(NetWebSocketConnectDescriptor::new(
            "ws://127.0.0.1:9/socket"
        ))
        .unwrap_err(),
        NetError::ProtocolUnavailable {
            capability: "runtime.feature.net.websocket".to_string(),
        }
    );
}
