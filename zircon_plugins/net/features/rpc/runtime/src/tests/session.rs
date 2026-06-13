use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetControlMessage, NetError, NetEvent,
    NetSessionHandshakePolicy, NetSessionHandshakeState, NetSessionId, NetTransportKind,
};

use super::support::{complete_joined_connection_session, process_hello, process_login};
use crate::{
    net_rpc_runtime_manager, NetRpcHandshakeFrame, NetRpcRuntimeManager,
    NET_RPC_FEATURE_CAPABILITY, RPC_HANDSHAKE_CAPABILITY_NET_RPC,
};

#[test]
fn rpc_feature_manager_completes_control_handshake_sequence() {
    let rpc = net_rpc_runtime_manager();
    let session = rpc.begin_handshake();

    let challenge = rpc
        .process_control_message(
            session,
            NetControlMessage::Hello {
                protocol_version: 1,
                runtime_features: vec![NET_RPC_FEATURE_CAPABILITY.to_string()],
            },
        )
        .unwrap();

    assert_eq!(challenge.session, session);
    assert_eq!(challenge.state, NetSessionHandshakeState::AwaitingLogin);
    assert_eq!(
        challenge.response,
        Some(NetControlMessage::Challenge {
            nonce: "zircon-rpc-challenge".to_string(),
        })
    );

    let welcome = rpc
        .process_control_message(
            session,
            NetControlMessage::Login {
                player_id: "player-one".to_string(),
                challenge_response: "zircon-rpc-challenge".to_string(),
            },
        )
        .unwrap();

    assert_eq!(welcome.state, NetSessionHandshakeState::Welcomed);
    assert_eq!(
        welcome.response,
        Some(NetControlMessage::Welcome {
            session_id: session.raw().to_string(),
            map: "zircon-default".to_string(),
        })
    );

    let netspeed = rpc
        .process_control_message(
            session,
            NetControlMessage::NetSpeed {
                bytes_per_second: 24_000,
            },
        )
        .unwrap();
    assert_eq!(netspeed.state, NetSessionHandshakeState::Welcomed);
    assert_eq!(netspeed.response, None);

    let joined = rpc
        .process_control_message(session, NetControlMessage::Join)
        .unwrap();
    assert_eq!(joined.state, NetSessionHandshakeState::Joined);
    assert_eq!(joined.response, None);
}

#[test]
fn rpc_feature_manager_records_session_connection_identity_and_netspeed() {
    let rpc = net_rpc_runtime_manager();
    let connection = NetConnectionId::new(42);
    let session = rpc.begin_handshake_for_connection(connection);

    let initial = rpc.session_info(session).unwrap();
    assert_eq!(initial.session, session);
    assert_eq!(initial.connection, Some(connection));
    assert_eq!(initial.player_id, None);
    assert_eq!(initial.netspeed_bytes_per_second, None);
    assert_eq!(initial.state, NetSessionHandshakeState::AwaitingHello);

    process_hello(&rpc, session);
    process_login(&rpc, session, "player-one");
    rpc.process_control_message(
        session,
        NetControlMessage::NetSpeed {
            bytes_per_second: 64_000,
        },
    )
    .unwrap();
    rpc.process_control_message(session, NetControlMessage::Join)
        .unwrap();

    let joined = rpc.session_info(session).unwrap();
    assert_eq!(joined.connection, Some(connection));
    assert_eq!(joined.player_id.as_deref(), Some("player-one"));
    assert_eq!(joined.netspeed_bytes_per_second, Some(64_000));
    assert_eq!(joined.state, NetSessionHandshakeState::Joined);
}

#[test]
fn rpc_feature_manager_closes_sessions_for_connection() {
    let rpc = net_rpc_runtime_manager();
    let closed_connection = NetConnectionId::new(9);
    let kept_connection = NetConnectionId::new(10);
    let closed_session = complete_joined_connection_session(&rpc, closed_connection, "closed");
    let kept_session = complete_joined_connection_session(&rpc, kept_connection, "kept");

    let closed = rpc.close_connection_sessions(closed_connection);
    assert_eq!(closed.len(), 1);
    assert_eq!(closed[0].session, closed_session);
    assert_eq!(closed[0].state, NetSessionHandshakeState::Closed);
    assert_eq!(
        rpc.handshake_state(closed_session).unwrap(),
        NetSessionHandshakeState::Closed
    );
    assert_eq!(
        rpc.handshake_state(kept_session).unwrap(),
        NetSessionHandshakeState::Joined
    );

    let explicitly_closed = rpc.close_session(kept_session).unwrap();
    assert_eq!(explicitly_closed.state, NetSessionHandshakeState::Closed);
    assert_eq!(
        rpc.close_session(NetSessionId::new(999)).unwrap_err(),
        NetError::UnknownSession {
            session: NetSessionId::new(999),
        }
    );
}

#[test]
fn rpc_feature_manager_closes_sessions_from_transport_events() {
    let rpc = net_rpc_runtime_manager();
    let closed_connection = NetConnectionId::new(21);
    let failed_connection = NetConnectionId::new(22);
    let ignored_connection = NetConnectionId::new(23);
    let closed_session = complete_joined_connection_session(&rpc, closed_connection, "closed");
    let failed_session = complete_joined_connection_session(&rpc, failed_connection, "failed");
    let ignored_session = complete_joined_connection_session(&rpc, ignored_connection, "ignored");

    let closed = rpc.apply_transport_events([
        NetEvent::ConnectionClosed {
            connection: closed_connection,
            transport: NetTransportKind::WebSocket,
        },
        NetEvent::ConnectionStateChanged {
            connection: failed_connection,
            transport: NetTransportKind::Tcp,
            state: NetConnectionState::Failed,
        },
        NetEvent::ConnectionStateChanged {
            connection: ignored_connection,
            transport: NetTransportKind::WebSocket,
            state: NetConnectionState::Open,
        },
    ]);

    let closed_sessions = closed.iter().map(|info| info.session).collect::<Vec<_>>();
    assert_eq!(closed_sessions, vec![closed_session, failed_session]);
    assert_eq!(
        rpc.handshake_state(closed_session).unwrap(),
        NetSessionHandshakeState::Closed
    );
    assert_eq!(
        rpc.handshake_state(failed_session).unwrap(),
        NetSessionHandshakeState::Closed
    );
    assert_eq!(
        rpc.handshake_state(ignored_session).unwrap(),
        NetSessionHandshakeState::Joined
    );
}

#[test]
fn rpc_feature_manager_reports_control_handshake_failures() {
    let rpc = NetRpcRuntimeManager::with_handshake_policy(
        NetSessionHandshakePolicy::new(2)
            .with_required_feature(NET_RPC_FEATURE_CAPABILITY)
            .with_challenge_nonce("challenge-v2"),
    );
    let session = rpc.begin_handshake();

    let report = rpc
        .process_control_message(
            session,
            NetControlMessage::Hello {
                protocol_version: 1,
                runtime_features: vec![NET_RPC_FEATURE_CAPABILITY.to_string()],
            },
        )
        .unwrap();

    assert_eq!(report.state, NetSessionHandshakeState::Failed);
    assert_eq!(
        report.response,
        Some(NetControlMessage::Failure {
            reason: "protocol version mismatch".to_string(),
        })
    );
    assert_eq!(
        rpc.handshake_state(session).unwrap(),
        NetSessionHandshakeState::Failed
    );
    assert_eq!(
        rpc.handshake_state(NetSessionId::new(999)).unwrap_err(),
        NetError::UnknownSession {
            session: NetSessionId::new(999),
        }
    );
}

#[test]
fn rpc_handshake_frame_round_trips_magic_version_capabilities_and_token() {
    let frame =
        NetRpcHandshakeFrame::new(1, RPC_HANDSHAKE_CAPABILITY_NET_RPC, b"join-token".to_vec());

    let encoded = frame.encode().unwrap();
    let decoded = NetRpcHandshakeFrame::decode(&encoded).unwrap();

    assert_eq!(decoded.protocol_version, 1);
    assert_eq!(decoded.capabilities, RPC_HANDSHAKE_CAPABILITY_NET_RPC);
    assert_eq!(decoded.token, b"join-token".to_vec());
    assert!(decoded.supports_net_rpc());
}

#[test]
fn handshake_version_mismatch_rejected() {
    let rpc = net_rpc_runtime_manager();
    let session = rpc.begin_handshake();
    let bytes =
        NetRpcHandshakeFrame::new(2, RPC_HANDSHAKE_CAPABILITY_NET_RPC, b"token-v2".to_vec())
            .encode()
            .unwrap();

    let report = rpc.process_handshake_frame(session, &bytes).unwrap();

    assert_eq!(report.state, NetSessionHandshakeState::Failed);
    assert_eq!(
        report.response,
        Some(NetControlMessage::Failure {
            reason: "protocol version mismatch".to_string(),
        })
    );
    assert_eq!(
        rpc.handshake_state(session).unwrap(),
        NetSessionHandshakeState::Failed
    );
}
