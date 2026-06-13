use zircon_plugin_net_runtime::DefaultNetManager;
use zircon_runtime::core::framework::net::NetManager;

use crate::websocket_runtime_manager;

#[test]
fn default_type_can_receive_websocket_backend_for_direct_tests() {
    let net: DefaultNetManager = websocket_runtime_manager();

    assert!(net.backend_name().contains("+websocket"));
}

#[test]
fn websocket_connection_send_path_is_queue_driven() {
    let source = include_str!("../backend/connection.rs");

    assert!(
        !source.contains("block_on"),
        "WebSocket frame sends must enqueue to the feature worker task instead of blocking the caller"
    );
    assert!(source.contains("mpsc::channel::<NetWebSocketFrame>"));
    assert!(source.contains(".try_send(frame)"));
}
