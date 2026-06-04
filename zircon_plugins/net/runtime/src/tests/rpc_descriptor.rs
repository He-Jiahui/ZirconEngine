use zircon_runtime::core::framework::net::{RpcDescriptor, RpcDirection};

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

    assert_eq!(
        RpcDescriptor::command("player.move").direction,
        RpcDirection::ClientToServer
    );
    assert_eq!(
        RpcDescriptor::client_rpc("chat.broadcast").direction,
        RpcDirection::ServerToClient
    );
    assert_eq!(
        RpcDescriptor::target_rpc("inventory.sync_one").direction,
        RpcDirection::TargetClient
    );
}
