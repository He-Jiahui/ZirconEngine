use zircon_runtime::core::framework::net::{NetConnectionId, NetControlMessage, NetSessionId};

use crate::{NetRpcRuntimeManager, NET_RPC_FEATURE_CAPABILITY};

pub(super) fn complete_joined_session(rpc: &NetRpcRuntimeManager, player_id: &str) -> NetSessionId {
    let session = rpc.begin_handshake();
    complete_existing_session(rpc, session, player_id);
    session
}

pub(super) fn complete_joined_connection_session(
    rpc: &NetRpcRuntimeManager,
    connection: NetConnectionId,
    player_id: &str,
) -> NetSessionId {
    let session = rpc.begin_handshake_for_connection(connection);
    complete_existing_session(rpc, session, player_id);
    session
}

fn complete_existing_session(rpc: &NetRpcRuntimeManager, session: NetSessionId, player_id: &str) {
    process_hello(rpc, session);
    process_login(rpc, session, player_id);
    rpc.process_control_message(session, NetControlMessage::Join)
        .unwrap();
}

pub(super) fn process_hello(rpc: &NetRpcRuntimeManager, session: NetSessionId) {
    rpc.process_control_message(
        session,
        NetControlMessage::Hello {
            protocol_version: 1,
            runtime_features: vec![NET_RPC_FEATURE_CAPABILITY.to_string()],
        },
    )
    .unwrap();
}

pub(super) fn process_login(rpc: &NetRpcRuntimeManager, session: NetSessionId, player_id: &str) {
    rpc.process_control_message(
        session,
        NetControlMessage::Login {
            player_id: player_id.to_string(),
            challenge_response: "zircon-rpc-challenge".to_string(),
        },
    )
    .unwrap();
}
