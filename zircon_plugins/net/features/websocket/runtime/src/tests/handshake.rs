use zircon_runtime::core::framework::net::{
    NetEndpoint, NetManager, NetWebSocketConnectDescriptor, NetWebSocketFrame,
    NetWebSocketListenerDescriptor,
};

use crate::websocket_runtime_manager;

use super::support::{
    accept_until_websocket, assert_websocket_policy_rejects, poll_websocket_frames_until_count,
    poll_websocket_until,
};

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
fn ws_frame_order_preserved() {
    let net = websocket_runtime_manager();
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
                "ws://{}:{}/ordered",
                endpoint.host, endpoint.port
            )))
            .unwrap()
    });
    let server = accept_until_websocket(&net, listener);
    let client = client_thread
        .join()
        .expect("websocket connect thread panicked");

    let client_frames = vec![
        NetWebSocketFrame::Text("client-one".to_string()),
        NetWebSocketFrame::Binary(vec![2]),
        NetWebSocketFrame::Text("client-three".to_string()),
        NetWebSocketFrame::Binary(vec![4]),
    ];
    for frame in client_frames.iter().cloned() {
        net.send_websocket_frame(client, frame).unwrap();
    }
    assert_eq!(
        poll_websocket_frames_until_count(&net, server, client_frames.len()),
        client_frames
    );

    let server_frames = vec![
        NetWebSocketFrame::Text("server-one".to_string()),
        NetWebSocketFrame::Binary(vec![12]),
        NetWebSocketFrame::Text("server-three".to_string()),
        NetWebSocketFrame::Binary(vec![14]),
    ];
    for frame in server_frames.iter().cloned() {
        net.send_websocket_frame(server, frame).unwrap();
    }
    assert_eq!(
        poll_websocket_frames_until_count(&net, client, server_frames.len()),
        server_frames
    );
}
