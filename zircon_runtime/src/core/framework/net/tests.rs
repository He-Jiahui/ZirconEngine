use super::*;
use zircon_runtime_interface::reflect::{ReflectSchemaRequest, ReflectTypePath};

#[test]
fn endpoint_transport_and_security_policy_are_neutral_contracts() {
    let endpoint = NetEndpoint::new("127.0.0.1", 0);

    assert_eq!(endpoint.to_string(), "127.0.0.1:0");
    assert_eq!(endpoint.to_socket_addr().unwrap().port(), 0);
    assert!(NetTransportKind::Tcp.is_tcp());
    assert!(!NetTransportKind::Udp.is_tcp());
    assert!(!NetTransportKind::Http.is_tcp());
    assert!(!NetTransportKind::WebSocket.is_tcp());
    assert!(!NetTransportKind::ReliableUdp.is_tcp());

    let policy =
        NetSecurityPolicy::production_tls().with_certificate_pin("api.zircon.dev", "sha256-demo");
    assert!(policy.tls_required);
    assert!(policy.certificate_pinning);
    assert!(!policy.allow_insecure_loopback);
    assert!(policy.has_pin_for_host("API.ZIRCON.DEV"));
    assert!(!policy.has_pin_for_host("cdn.zircon.dev"));

    let rooted = NetSecurityPolicy::development().with_certificate_root_der(vec![1, 2, 3]);
    assert!(rooted.has_certificate_roots());
    assert_eq!(rooted.certificate_roots[0].der, vec![1, 2, 3]);

    let json = serde_json::to_value(&policy).unwrap();
    assert_eq!(
        serde_json::from_value::<NetSecurityPolicy>(json).unwrap(),
        policy
    );

    let diagnostics = NetDiagnostics {
        backend_name: "tokio-net+http+websocket".to_string(),
        mode: NetRuntimeMode::DedicatedServer,
        outbound_bytes: 42,
        inbound_bytes: 24,
        last_observed_latency_ms: Some(16),
        open_udp_sockets: 1,
        open_tcp_listeners: 2,
        open_http_listeners: 3,
        open_websocket_listeners: 4,
        open_tcp_connections: 5,
        open_http_routes: 6,
        open_websocket_connections: 7,
        queued_events: 8,
    };
    let json = serde_json::to_value(&diagnostics).unwrap();
    assert_eq!(
        serde_json::from_value::<NetDiagnostics>(json).unwrap(),
        diagnostics
    );
}

#[test]
fn http_and_websocket_descriptors_keep_protocol_state_data_only() {
    let request = NetHttpRequestDescriptor::new(
        NetRequestId::new(7),
        NetHttpMethod::Post,
        "https://api.zircon.dev/rpc",
    )
    .with_header("content-type", "application/json")
    .with_body(br#"{"op":"ping"}"#.to_vec())
    .with_max_retry_attempts(3);

    assert_eq!(request.request, NetRequestId::new(7));
    assert_eq!(request.method, NetHttpMethod::Post);
    assert_eq!(request.timeout_ms, 30_000);
    assert_eq!(request.max_retry_attempts, 3);
    assert_eq!(
        request.headers[0],
        ("content-type".into(), "application/json".into())
    );
    assert_eq!(request.body, br#"{"op":"ping"}"#);
    assert_eq!(request.security, NetSecurityPolicy::development());

    let ranged = NetHttpRequestDescriptor::new(
        NetRequestId::new(8),
        NetHttpMethod::Get,
        "https://cdn.zircon.dev/chunk.bin",
    )
    .with_header("range", "bytes=0-15")
    .with_byte_range(16, 31);
    assert_eq!(ranged.headers, vec![("range".into(), "bytes=16-31".into())]);

    let response = NetHttpResponseDescriptor::new(NetRequestId::new(0), 202, b"ok".to_vec())
        .with_header("x-request", "accepted")
        .for_request(NetRequestId::new(7));
    assert_eq!(response.request, NetRequestId::new(7));
    assert_eq!(response.status_code, 202);
    assert_eq!(response.body_bytes, 2);
    assert_eq!(response.headers[0], ("x-request".into(), "accepted".into()));

    let route = NetHttpRouteDescriptor::new("/rpc", [NetHttpMethod::Post, NetHttpMethod::Patch]);
    assert_eq!(route.path, "/rpc");
    assert_eq!(
        route.methods,
        vec![NetHttpMethod::Post, NetHttpMethod::Patch]
    );
    assert_eq!(route.endpoint, None);

    let listener = NetWebSocketListenerDescriptor::new(NetEndpoint::new("127.0.0.1", 9001))
        .with_allowed_path("/ws")
        .with_required_header("x-zr-session", "demo")
        .with_allowed_protocol("zrnet.v1");
    assert_eq!(listener.bind, NetEndpoint::new("127.0.0.1", 9001));
    assert_eq!(listener.allowed_paths, vec!["/ws"]);
    assert_eq!(
        listener.required_headers,
        vec![("x-zr-session".into(), "demo".into())]
    );
    assert_eq!(listener.allowed_protocols, vec!["zrnet.v1"]);

    let connect = NetWebSocketConnectDescriptor::new("wss://api.zircon.dev/ws")
        .with_header("authorization", "Bearer test")
        .with_protocol("zrnet.v1");
    assert_eq!(connect.timeout_ms, 30_000);
    assert_eq!(connect.security, NetSecurityPolicy::development());
    assert_eq!(connect.protocols, vec!["zrnet.v1"]);

    let close = NetWebSocketFrame::Close(NetWebSocketCloseReason::normal("done"));
    let json = serde_json::to_value(&close).unwrap();
    assert_eq!(
        serde_json::from_value::<NetWebSocketFrame>(json).unwrap(),
        close
    );

    let events = vec![
        NetEvent::UdpSocketClosed {
            socket: NetSocketId::new(2),
        },
        NetEvent::ListenerClosed {
            listener: NetListenerId::new(3),
            transport: NetTransportKind::Http,
        },
        NetEvent::ConnectionAccepted {
            listener: NetListenerId::new(8),
            connection: NetConnectionId::new(9),
            transport: NetTransportKind::WebSocket,
            remote: NetEndpoint::new("127.0.0.1", 9010),
        },
        NetEvent::ConnectionClosed {
            connection: NetConnectionId::new(9),
            transport: NetTransportKind::WebSocket,
        },
        NetEvent::HttpRouteUnregistered {
            route: NetRouteId::new(4),
        },
    ];
    let json = serde_json::to_value(&events).unwrap();
    assert_eq!(
        serde_json::from_value::<Vec<NetEvent>>(json).unwrap(),
        events
    );
}

#[test]
fn rpc_session_and_handshake_descriptors_are_runtime_mode_agnostic() {
    assert!(RpcDirection::ClientToServer.allows_caller(RpcPeerRole::Client));
    assert!(!RpcDirection::ClientToServer.allows_caller(RpcPeerRole::Server));
    assert!(RpcDirection::ServerToClient.allows_caller(RpcPeerRole::Server));
    assert!(RpcDirection::TargetClient.allows_caller(RpcPeerRole::Server));
    assert!(RpcDirection::Bidirectional.allows_caller(RpcPeerRole::Client));
    assert!(RpcDirection::Bidirectional.allows_caller(RpcPeerRole::Server));

    let descriptor = RpcDescriptor::target_rpc("inventory.sync_one")
        .with_payload_schema("schema://net/inventory/sync-one.v1")
        .with_max_calls_per_second(12)
        .with_max_payload_bytes(4096);
    assert_eq!(descriptor.direction, RpcDirection::TargetClient);
    assert_eq!(
        descriptor
            .payload_schema
            .as_ref()
            .map(|schema| schema.schema_id()),
        Some("schema://net/inventory/sync-one.v1")
    );
    assert_eq!(
        descriptor
            .payload_schema
            .as_ref()
            .map(|schema| schema.reflect_schema_request.clone()),
        Some(ReflectSchemaRequest::for_type(
            "schema://net/inventory/sync-one.v1"
        ))
    );
    assert_eq!(descriptor.max_calls_per_second, Some(12));
    assert_eq!(descriptor.max_payload_bytes, Some(4096));
    assert!(RpcDirection::Bidirectional
        .allows_invocation(RpcDirection::ClientToServer, RpcPeerRole::Client));
    assert!(RpcDirection::Bidirectional
        .allows_invocation(RpcDirection::ServerToClient, RpcPeerRole::Server));
    assert!(!RpcDirection::Bidirectional
        .allows_invocation(RpcDirection::ServerToClient, RpcPeerRole::Client));

    let reflected_schema = RpcPayloadSchema::from_reflect_type_path(
        ReflectTypePath::new("gameplay.inventory.SyncOne", "SyncOne")
            .unwrap()
            .with_plugin_id("net"),
    );
    assert_eq!(reflected_schema.schema_id(), "gameplay.inventory.SyncOne");
    assert_eq!(
        reflected_schema.reflect_schema_request,
        ReflectSchemaRequest::for_type("gameplay.inventory.SyncOne")
    );

    let invocation = RpcInvocationDescriptor::new(
        "inventory.sync_one",
        RpcDirection::TargetClient,
        vec![1, 2, 3, 4],
    )
    .with_request(NetRequestId::new(11))
    .with_source_session(NetSessionId::new(2))
    .with_target_session(NetSessionId::new(3))
    .with_timeout_ms(250)
    .with_priority(9);
    assert_eq!(invocation.payload_bytes(), 4);

    let report = RpcDispatchReport::for_invocation(&invocation, RpcDispatchStatus::Queued)
        .with_schema(
            descriptor
                .payload_schema
                .as_ref()
                .map(|schema| schema.schema_id.clone()),
        )
        .with_diagnostic("queued for reliable channel")
        .with_response_payload(vec![9]);
    assert_eq!(report.rpc_id, "inventory.sync_one");
    assert_eq!(report.request, Some(NetRequestId::new(11)));
    assert_eq!(report.status, RpcDispatchStatus::Queued);
    assert_eq!(report.payload_bytes, 4);
    assert_eq!(report.response_payload, Some(vec![9]));

    let policy = NetSessionHandshakePolicy::new(4)
        .with_required_feature("runtime.feature.net.rpc")
        .with_challenge_nonce("nonce-1")
        .with_welcome_map("arena");
    assert_eq!(policy.protocol_version, 4);
    assert_eq!(policy.required_features, vec!["runtime.feature.net.rpc"]);
    assert_eq!(policy.challenge_nonce, "nonce-1");
    assert_eq!(policy.welcome_map, "arena");

    let session = NetSessionInfo::new(
        NetSessionId::new(3),
        Some(NetConnectionId::new(8)),
        NetSessionHandshakeState::Welcomed,
        Some("player-a".to_string()),
        Some(16_384),
    );
    assert_eq!(session.player_id.as_deref(), Some("player-a"));

    let control = NetSessionControlReport::new(
        NetSessionId::new(3),
        NetSessionHandshakeState::Joined,
        Some(NetControlMessage::Welcome {
            session_id: "session-3".to_string(),
            map: "arena".to_string(),
        }),
    );
    let json = serde_json::to_value(&control).unwrap();
    assert_eq!(
        serde_json::from_value::<NetSessionControlReport>(json).unwrap(),
        control
    );
}

#[test]
fn reliable_datagram_and_download_contracts_record_recovery_state() {
    let config = ReliableDatagramConfig::default();
    assert_eq!(config.mtu_bytes, 1_200);
    assert_eq!(config.resend_timeout_ms, 100);
    assert_eq!(config.max_resend_attempts, 8);
    assert_eq!(config.receive_window, 256);

    let simulation = ReliableDatagramSimulationProfile::new()
        .with_drop_every_nth_packet(3)
        .with_reorder_window(0)
        .with_recovery_drop_threshold(5);
    assert_eq!(simulation.drop_every_nth_packet, Some(3));
    assert_eq!(simulation.reorder_window, 1);
    assert_eq!(simulation.recovery_drop_threshold, Some(5));

    let packet = ReliableDatagramPacket::new(42, "state", vec![1, 2, 3]).with_fragment(1, 3);
    assert_eq!(packet.fragment_index, 1);
    assert_eq!(packet.fragment_count, 3);

    let receive =
        ReliableDatagramReceiveReport::new(42, "state", ReliableDatagramReceiveStatus::Reassembled)
            .with_ack(ReliableDatagramAck::new(42))
            .with_payload(vec![1, 2, 3])
            .with_diagnostic("reassembled");
    assert_eq!(receive.ack, Some(ReliableDatagramAck::new(42)));
    assert_eq!(receive.payload, Some(vec![1, 2, 3]));

    let recovery =
        ReliableDatagramRecoveryReport::new(ReliableDatagramRecoveryState::Recovering, 2, 5)
            .with_diagnostic("resending");
    let delivery =
        ReliableDatagramDeliveryReport::new([packet.clone()], std::iter::empty(), recovery);
    assert_eq!(delivery.delivered_packets, vec![packet.clone()]);
    assert!(delivery.dropped_packets.is_empty());

    let send = ReliableDatagramSendReport::new(ReliableDatagramSendStatus::Fragmented, [packet]);
    assert_eq!(send.status, ReliableDatagramSendStatus::Fragmented);
    assert_eq!(send.packets.len(), 1);

    let chunk = NetDownloadChunk::new(
        "chunk-0",
        "https://cdn.zircon.dev/world.bin",
        0,
        1024,
        [7; 32],
    )
    .with_resume_from_byte(512);
    assert_eq!(chunk.resume_from_byte, Some(512));
    assert!(chunk.allow_range_resume);

    let manifest = NetDownloadManifest::new(NetDownloadId::new(9), "asset://world")
        .with_chunk(chunk)
        .with_mirror_url("https://mirror.zircon.dev/world.bin");
    assert_eq!(manifest.chunks.len(), 1);
    assert_eq!(manifest.mirror_urls.len(), 1);

    let progress =
        NetDownloadProgress::new(NetDownloadId::new(9), NetDownloadStatus::Verifying, 1024)
            .with_diagnostic("hash pending");
    assert_eq!(progress.status, NetDownloadStatus::Verifying);
    assert_eq!(progress.diagnostic.as_deref(), Some("hash pending"));
}

#[test]
fn sync_descriptors_share_interest_budget_and_delta_contracts() {
    let identity = NetworkIdentity::new(NetObjectId::new(21), SyncAuthority::Server);
    assert_eq!(identity.object, NetObjectId::new(21));
    assert_eq!(identity.authority, SyncAuthority::Server);

    let descriptor = SyncComponentDescriptor::new("Transform", SyncAuthority::Server)
        .with_field(SyncFieldDescriptor::new("translation", "vec3").delta_compressed(false))
        .with_field(SyncFieldDescriptor::new("rotation", "quat"))
        .with_replication_strategy(SyncReplicationStrategy::Interval)
        .with_update_hz(30)
        .with_replication_priority(7)
        .with_interest_group("nearby");

    assert_eq!(
        descriptor.replication_strategy,
        SyncReplicationStrategy::Interval
    );
    assert_eq!(descriptor.update_hz, 30);
    assert_eq!(descriptor.replication_priority, 7);
    assert_eq!(descriptor.interest_group.as_deref(), Some("nearby"));
    assert!(!descriptor.fields[0].delta_compressed);
    assert!(descriptor.fields[1].delta_compressed);

    let snapshot = SyncObjectSnapshot::new(
        NetObjectId::new(21),
        &descriptor,
        [
            SyncFieldValue::new("translation", [1, 2, 3]),
            SyncFieldValue::new("rotation", [0, 0, 0, 1]),
        ],
    );
    assert_eq!(snapshot.object, NetObjectId::new(21));
    assert_eq!(snapshot.component_type, "Transform");
    assert_eq!(snapshot.interest_group.as_deref(), Some("nearby"));
    assert_eq!(snapshot.fields.len(), 2);

    let delta = SyncDelta::new(
        NetObjectId::new(21),
        "Transform",
        99,
        [SyncFieldValue::new("translation", [4, 5, 6])],
    );
    assert_eq!(delta.sequence, 99);
    assert!(!delta.is_despawn());
    assert_eq!(delta.changed_fields.len(), 1);

    let despawn_delta = SyncDelta::despawn(NetObjectId::new(21), "Transform", 100);
    assert_eq!(despawn_delta.sequence, 100);
    assert!(despawn_delta.is_despawn());
    assert!(despawn_delta.changed_fields.is_empty());

    let interest = SyncInterestDescriptor::new(NetSessionId::new(3)).with_group("nearby");
    assert!(interest.allows_group(Some("nearby")));
    assert!(!interest.allows_group(Some("far")));
    assert!(interest.allows_group(None));

    let budget = SyncReplicationBudget::new()
        .with_max_snapshots(2)
        .with_max_bytes(16);
    assert!(budget.allows_snapshot_count(1));
    assert!(!budget.allows_snapshot_count(2));
    assert!(budget.allows_byte_count(8, 8));
    assert!(!budget.allows_byte_count(12, 8));

    let schedule = SyncReplicationScheduleReport::new(NetSessionId::new(3), 16, budget);
    assert_eq!(schedule.sent_snapshots.len(), 0);
    assert_eq!(schedule.used_bytes, 0);
    assert_eq!(schedule.deferred_snapshots, 0);
}
