use zircon_runtime::core::framework::net::{
    NetControlMessage, RpcDescriptor, RpcDirection, RpcDispatchStatus, RpcInvocationDescriptor,
    RpcPeerRole,
};

use super::support::complete_joined_session;
use crate::net_rpc_runtime_manager;

#[test]
fn rpc_feature_manager_validates_registry_authority_payload_and_quota() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "player-seven");
    let payload_session = complete_joined_session(&rpc, "player-eight");
    rpc.register_rpc(
        RpcDescriptor::new("chat.send", RpcDirection::ClientToServer)
            .with_max_calls_per_second(1)
            .with_max_payload_bytes(8),
    )
    .unwrap();
    rpc.register_rpc(RpcDescriptor::new(
        "chat.notice",
        RpcDirection::ServerToClient,
    ))
    .unwrap();

    let first = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.send", RpcDirection::ClientToServer, b"ping".to_vec())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(first.status, RpcDispatchStatus::Accepted);

    let quota_block = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.send", RpcDirection::ClientToServer, b"pong".to_vec())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(quota_block.status, RpcDispatchStatus::QuotaExceeded);

    let payload_block = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new(
            "chat.send",
            RpcDirection::ClientToServer,
            b"too-large".to_vec(),
        )
        .with_source_session(payload_session),
        RpcPeerRole::Client,
    );
    assert_eq!(payload_block.status, RpcDispatchStatus::PayloadTooLarge);

    let authority_block = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.send", RpcDirection::ClientToServer, b"ping".to_vec()),
        RpcPeerRole::Server,
    );
    assert_eq!(authority_block.status, RpcDispatchStatus::DirectionDenied);

    let no_handler = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("missing.rpc", RpcDirection::ClientToServer, Vec::new()),
        RpcPeerRole::Client,
    );
    assert_eq!(no_handler.status, RpcDispatchStatus::NoHandler);
}

#[test]
fn wrong_direction_rpc_rejected() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "direction-player");
    rpc.register_rpc(RpcDescriptor::new(
        "chat.notice",
        RpcDirection::ServerToClient,
    ))
    .unwrap();

    let wrong_payload_direction = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.notice", RpcDirection::ClientToServer, Vec::new())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(
        wrong_payload_direction.status,
        RpcDispatchStatus::DirectionDenied
    );

    let wrong_caller_role = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.notice", RpcDirection::ServerToClient, Vec::new()),
        RpcPeerRole::Client,
    );
    assert_eq!(wrong_caller_role.status, RpcDispatchStatus::DirectionDenied);
}

#[test]
fn bidirectional_rpc_accepts_valid_client_and_server_calls() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "bidirectional-player");
    rpc.register_rpc(RpcDescriptor::new(
        "chat.moderate",
        RpcDirection::Bidirectional,
    ))
    .unwrap();

    let client_to_server = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.moderate", RpcDirection::ClientToServer, Vec::new())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(client_to_server.status, RpcDispatchStatus::Accepted);

    let server_to_client = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.moderate", RpcDirection::ServerToClient, Vec::new()),
        RpcPeerRole::Server,
    );
    assert_eq!(server_to_client.status, RpcDispatchStatus::Accepted);

    let mismatched_role = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.moderate", RpcDirection::ServerToClient, Vec::new()),
        RpcPeerRole::Client,
    );
    assert_eq!(mismatched_role.status, RpcDispatchStatus::DirectionDenied);
}

#[test]
fn rpc_feature_manager_denies_client_rpc_until_source_session_joined() {
    let rpc = net_rpc_runtime_manager();
    rpc.register_rpc(RpcDescriptor::command("chat.send"))
        .unwrap();
    let pending_session = rpc.begin_handshake();
    let joined_session = complete_joined_session(&rpc, "joined-player");

    let missing_source = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.send", RpcDirection::ClientToServer, b"ping".to_vec()),
        RpcPeerRole::Client,
    );
    assert_eq!(missing_source.status, RpcDispatchStatus::SessionUnavailable);
    assert_eq!(
        missing_source.diagnostic.as_deref(),
        Some("client-to-server RPC requires a source session")
    );

    let before_join = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.send", RpcDirection::ClientToServer, b"ping".to_vec())
            .with_source_session(pending_session),
        RpcPeerRole::Client,
    );
    assert_eq!(before_join.status, RpcDispatchStatus::SessionUnavailable);
    assert_eq!(
        before_join.diagnostic.as_deref(),
        Some("source session is not joined")
    );

    let accepted = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.send", RpcDirection::ClientToServer, b"ping".to_vec())
            .with_source_session(joined_session),
        RpcPeerRole::Client,
    );
    assert_eq!(accepted.status, RpcDispatchStatus::Accepted);

    rpc.close_session(joined_session).unwrap();
    let after_close = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.send", RpcDirection::ClientToServer, b"ping".to_vec())
            .with_source_session(joined_session),
        RpcPeerRole::Client,
    );
    assert_eq!(after_close.status, RpcDispatchStatus::SessionUnavailable);
    assert_eq!(
        after_close.diagnostic.as_deref(),
        Some("source session is closed")
    );
}

#[test]
fn rpc_feature_manager_applies_netspeed_byte_budget_before_call_quota() {
    let rpc = net_rpc_runtime_manager();
    rpc.register_rpc(
        RpcDescriptor::command("chat.send")
            .with_max_calls_per_second(4)
            .with_max_payload_bytes(16),
    )
    .unwrap();
    let session = complete_joined_session(&rpc, "budgeted-player");
    rpc.process_control_message(
        session,
        NetControlMessage::NetSpeed {
            bytes_per_second: 4,
        },
    )
    .unwrap();

    let accepted = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.send", RpcDirection::ClientToServer, b"ping".to_vec())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(accepted.status, RpcDispatchStatus::Accepted);

    let budget_block = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.send", RpcDirection::ClientToServer, b"x".to_vec())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(budget_block.status, RpcDispatchStatus::QuotaExceeded);
    assert_eq!(
        budget_block.diagnostic.as_deref(),
        Some("source session NetSpeed byte budget exceeded")
    );
}
