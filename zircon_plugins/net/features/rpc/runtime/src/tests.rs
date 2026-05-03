use zircon_runtime::core::framework::net::{
    NetConnectionId, NetConnectionState, NetControlMessage, NetError, NetEvent, NetRequestId,
    NetSessionHandshakePolicy, NetSessionHandshakeState, NetSessionId, NetTransportKind,
    RpcDescriptor, RpcDirection, RpcDispatchStatus, RpcInvocationDescriptor, RpcPeerRole,
};

use super::{
    net_rpc_runtime_manager, plugin_feature_registration, NetRpcRuntimeManager,
    NET_RPC_FEATURE_CAPABILITY, NET_RPC_FEATURE_ID, NET_RPC_FEATURE_MANAGER_NAME,
    NET_RPC_FEATURE_MODULE_NAME,
};

#[test]
fn rpc_feature_registration_contributes_runtime_module_and_manager() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, NET_RPC_FEATURE_ID);
    assert!(report
        .manifest
        .capabilities
        .iter()
        .any(|capability| capability == NET_RPC_FEATURE_CAPABILITY));
    let module = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NET_RPC_FEATURE_MODULE_NAME)
        .expect("RPC feature module should be registered");
    assert_eq!(
        module.managers[0].name.to_string(),
        NET_RPC_FEATURE_MANAGER_NAME
    );
}

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

#[test]
fn rpc_feature_manager_validates_schema_then_invokes_handler() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "schema-player");
    rpc.register_schema_validator("schema://chat/echo.v1", |payload| {
        payload.starts_with(b"ok:")
    });
    rpc.register_rpc_handler(
        RpcDescriptor::command("chat.echo")
            .with_payload_schema("schema://chat/echo.v1")
            .with_max_payload_bytes(32),
        |invocation| Ok(invocation.payload.clone()),
    )
    .unwrap();

    let accepted = rpc.invoke_rpc(
        RpcInvocationDescriptor::new(
            "chat.echo",
            RpcDirection::ClientToServer,
            b"ok:ping".to_vec(),
        )
        .with_source_session(session),
        RpcPeerRole::Client,
    );

    assert_eq!(accepted.status, RpcDispatchStatus::Accepted);
    assert_eq!(accepted.response_payload, Some(b"ok:ping".to_vec()));
    assert_eq!(accepted.schema, Some("schema://chat/echo.v1".to_string()));

    let rejected = rpc.invoke_rpc(
        RpcInvocationDescriptor::new("chat.echo", RpcDirection::ClientToServer, b"bad".to_vec())
            .with_source_session(session),
        RpcPeerRole::Client,
    );

    assert_eq!(rejected.status, RpcDispatchStatus::SchemaRejected);
    assert_eq!(rejected.response_payload, None);
}

#[test]
fn rpc_feature_manager_reports_schema_handler_and_missing_handler_failures() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "handler-player");
    rpc.register_rpc(RpcDescriptor::command("chat.no_handler"))
        .unwrap();
    rpc.register_rpc(
        RpcDescriptor::command("chat.schema_missing")
            .with_payload_schema("schema://chat/missing.v1"),
    )
    .unwrap();
    rpc.register_schema_validator("schema://chat/fail.v1", |_| true);
    rpc.register_rpc_handler(
        RpcDescriptor::command("chat.handler_fails").with_payload_schema("schema://chat/fail.v1"),
        |_| Err("handler exploded".to_string()),
    )
    .unwrap();

    let no_handler = rpc.invoke_rpc(
        RpcInvocationDescriptor::new("chat.no_handler", RpcDirection::ClientToServer, Vec::new())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(no_handler.status, RpcDispatchStatus::NoHandler);

    let schema_missing = rpc.invoke_rpc(
        RpcInvocationDescriptor::new(
            "chat.schema_missing",
            RpcDirection::ClientToServer,
            Vec::new(),
        )
        .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(schema_missing.status, RpcDispatchStatus::SchemaUnavailable);
    assert_eq!(
        schema_missing.diagnostic.as_deref(),
        Some("schema validator unavailable")
    );

    let handler_failed = rpc.invoke_rpc(
        RpcInvocationDescriptor::new(
            "chat.handler_fails",
            RpcDirection::ClientToServer,
            Vec::new(),
        )
        .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(handler_failed.status, RpcDispatchStatus::HandlerFailed);
    assert_eq!(
        handler_failed.diagnostic.as_deref(),
        Some("handler exploded")
    );
}

#[test]
fn rpc_feature_manager_correlates_requests_and_drains_priority_queue() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "queued-player");
    rpc.register_rpc_handler(RpcDescriptor::command("chat.echo"), |invocation| {
        Ok(invocation.payload.clone())
    })
    .unwrap();

    let low = rpc.enqueue_rpc(
        RpcInvocationDescriptor::new("chat.echo", RpcDirection::ClientToServer, b"low".to_vec())
            .with_source_session(session)
            .with_request(NetRequestId::new(10))
            .with_priority(1),
        RpcPeerRole::Client,
    );
    assert_eq!(low.status, RpcDispatchStatus::Queued);
    assert_eq!(low.request, Some(NetRequestId::new(10)));

    let high = rpc.enqueue_rpc(
        RpcInvocationDescriptor::new("chat.echo", RpcDirection::ClientToServer, b"high".to_vec())
            .with_source_session(session)
            .with_request(NetRequestId::new(11))
            .with_priority(9),
        RpcPeerRole::Client,
    );
    assert_eq!(high.status, RpcDispatchStatus::Queued);

    let drained = rpc.drain_rpc_queue(4);
    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].request, Some(NetRequestId::new(11)));
    assert_eq!(drained[0].response_payload, Some(b"high".to_vec()));
    assert_eq!(drained[1].request, Some(NetRequestId::new(10)));
    assert_eq!(drained[1].response_payload, Some(b"low".to_vec()));
}

#[test]
fn rpc_feature_manager_limits_queue_and_times_out_expired_invocations() {
    let rpc = NetRpcRuntimeManager::with_max_queue_depth(1);
    let session = complete_joined_session(&rpc, "queued-player");
    rpc.register_rpc_handler(RpcDescriptor::command("chat.echo"), |invocation| {
        Ok(invocation.payload.clone())
    })
    .unwrap();

    let queued = rpc.enqueue_rpc(
        RpcInvocationDescriptor::new("chat.echo", RpcDirection::ClientToServer, b"one".to_vec())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(queued.status, RpcDispatchStatus::Queued);

    let queue_full = rpc.enqueue_rpc(
        RpcInvocationDescriptor::new("chat.echo", RpcDirection::ClientToServer, b"two".to_vec())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(queue_full.status, RpcDispatchStatus::QueueFull);

    let timed_out = rpc.invoke_rpc(
        RpcInvocationDescriptor::new("chat.echo", RpcDirection::ClientToServer, b"late".to_vec())
            .with_source_session(session)
            .with_timeout_ms(0),
        RpcPeerRole::Client,
    );
    assert_eq!(timed_out.status, RpcDispatchStatus::TimedOut);
    assert_eq!(timed_out.response_payload, None);
}

#[test]
fn rpc_feature_manager_rejects_queue_full_before_charging_call_quota() {
    let rpc = NetRpcRuntimeManager::with_max_queue_depth(1);
    let session = complete_joined_session(&rpc, "quota-player");
    rpc.register_rpc_handler(
        RpcDescriptor::command("chat.once").with_max_calls_per_second(1),
        |invocation| Ok(invocation.payload.clone()),
    )
    .unwrap();

    let queued = rpc.enqueue_rpc(
        RpcInvocationDescriptor::new("chat.once", RpcDirection::ClientToServer, b"one".to_vec())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    let queue_full = rpc.enqueue_rpc(
        RpcInvocationDescriptor::new("chat.once", RpcDirection::ClientToServer, b"two".to_vec())
            .with_source_session(session),
        RpcPeerRole::Client,
    );

    assert_eq!(queued.status, RpcDispatchStatus::Queued);
    assert_eq!(queue_full.status, RpcDispatchStatus::QueueFull);
    let direct = rpc.invoke_rpc(
        RpcInvocationDescriptor::new(
            "chat.once",
            RpcDirection::ClientToServer,
            b"direct".to_vec(),
        )
        .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(direct.status, RpcDispatchStatus::QuotaExceeded);
}

#[test]
fn rpc_feature_manager_drains_queued_rpc_without_double_counting_admission_quota() {
    let rpc = NetRpcRuntimeManager::with_max_queue_depth(4);
    let session = complete_joined_session(&rpc, "quota-player");
    rpc.register_rpc_handler(
        RpcDescriptor::command("chat.once").with_max_calls_per_second(1),
        |invocation| Ok(invocation.payload.clone()),
    )
    .unwrap();

    let queued = rpc.enqueue_rpc(
        RpcInvocationDescriptor::new("chat.once", RpcDirection::ClientToServer, b"one".to_vec())
            .with_source_session(session),
        RpcPeerRole::Client,
    );

    assert_eq!(queued.status, RpcDispatchStatus::Queued);
    let drained = rpc.drain_rpc_queue(4);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].status, RpcDispatchStatus::Accepted);
    assert_eq!(drained[0].response_payload, Some(b"one".to_vec()));
}

#[test]
fn rpc_feature_manager_marks_expired_queued_rpc_timed_out_without_handler_call() {
    let rpc = NetRpcRuntimeManager::with_max_queue_depth(4);
    let session = complete_joined_session(&rpc, "timeout-player");
    rpc.register_rpc_handler(RpcDescriptor::command("chat.expire"), |_| {
        Ok(b"unexpected".to_vec())
    })
    .unwrap();

    let queued = rpc.enqueue_rpc(
        RpcInvocationDescriptor::new("chat.expire", RpcDirection::ClientToServer, Vec::new())
            .with_source_session(session)
            .with_timeout_ms(0),
        RpcPeerRole::Client,
    );

    assert_eq!(queued.status, RpcDispatchStatus::Queued);
    let drained = rpc.drain_rpc_queue(4);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].status, RpcDispatchStatus::TimedOut);
    assert_eq!(drained[0].response_payload, None);
}

fn complete_joined_session(rpc: &NetRpcRuntimeManager, player_id: &str) -> NetSessionId {
    let session = rpc.begin_handshake();
    complete_existing_session(rpc, session, player_id);
    session
}

fn complete_joined_connection_session(
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

fn process_hello(rpc: &NetRpcRuntimeManager, session: NetSessionId) {
    rpc.process_control_message(
        session,
        NetControlMessage::Hello {
            protocol_version: 1,
            runtime_features: vec![NET_RPC_FEATURE_CAPABILITY.to_string()],
        },
    )
    .unwrap();
}

fn process_login(rpc: &NetRpcRuntimeManager, session: NetSessionId, player_id: &str) {
    rpc.process_control_message(
        session,
        NetControlMessage::Login {
            player_id: player_id.to_string(),
            challenge_response: "zircon-rpc-challenge".to_string(),
        },
    )
    .unwrap();
}
