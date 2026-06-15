use std::thread;
use std::time::Duration;

use zircon_runtime::core::framework::net::{
    NetRequestId, RpcDescriptor, RpcDirection, RpcDispatchStatus, RpcInvocationDescriptor,
    RpcPeerRole,
};

use super::support::complete_joined_session;
use crate::net_rpc_runtime_manager;

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
fn rpc_payload_schema_uses_reflect_schema_request() {
    let rpc = net_rpc_runtime_manager();
    rpc.register_rpc(
        RpcDescriptor::command("chat.schema").with_payload_schema("gameplay.ChatMessage"),
    )
    .unwrap();

    let descriptor = rpc.rpc_descriptor("chat.schema").unwrap();
    let schema = descriptor.payload_schema.as_ref().unwrap();
    assert_eq!(schema.schema_id(), "gameplay.ChatMessage");
    assert_eq!(
        schema.reflect_schema_request.filter.type_path.as_deref(),
        Some("gameplay.ChatMessage")
    );
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
fn rpc_feature_manager_tracks_and_completes_correlated_pending_request() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "pending-player");
    rpc.register_rpc_handler(RpcDescriptor::command("chat.pending"), |invocation| {
        Ok(invocation.payload.clone())
    })
    .unwrap();

    let request = NetRequestId::new(77);
    assert!(rpc.pending_request(request).is_none());
    let report = rpc.invoke_rpc(
        RpcInvocationDescriptor::new(
            "chat.pending",
            RpcDirection::ClientToServer,
            b"body".to_vec(),
        )
        .with_source_session(session)
        .with_request(request)
        .with_timeout_ms(1000),
        RpcPeerRole::Client,
    );

    assert_eq!(report.status, RpcDispatchStatus::Accepted);
    assert_eq!(report.request, Some(request));
    assert_eq!(report.response_payload, Some(b"body".to_vec()));
    assert!(rpc.pending_request(request).is_none());
}

#[test]
fn rpc_feature_manager_expires_correlated_pending_requests() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "pending-player");
    rpc.register_rpc(RpcDescriptor::command("chat.no_handler"))
        .unwrap();

    let request = NetRequestId::new(78);
    let report = rpc.invoke_rpc(
        RpcInvocationDescriptor::new("chat.no_handler", RpcDirection::ClientToServer, Vec::new())
            .with_source_session(session)
            .with_request(request)
            .with_timeout_ms(0),
        RpcPeerRole::Client,
    );
    assert_eq!(report.status, RpcDispatchStatus::TimedOut);
    assert!(rpc.pending_request(request).is_none());
    assert!(rpc.expire_pending_requests().is_empty());
}

#[test]
fn rpc_feature_manager_marks_slow_handler_timed_out_without_response_payload() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "timeout-player");
    rpc.register_rpc_handler(RpcDescriptor::command("chat.slow"), |_| {
        thread::sleep(Duration::from_millis(50));
        Ok(b"late".to_vec())
    })
    .unwrap();

    let request = NetRequestId::new(91);
    let timed_out = rpc.invoke_rpc(
        RpcInvocationDescriptor::new("chat.slow", RpcDirection::ClientToServer, Vec::new())
            .with_source_session(session)
            .with_request(request)
            .with_timeout_ms(10),
        RpcPeerRole::Client,
    );

    assert_eq!(timed_out.status, RpcDispatchStatus::TimedOut);
    assert_eq!(timed_out.response_payload, None);
    assert_eq!(
        timed_out.diagnostic.as_deref(),
        Some("RPC handler exceeded invocation timeout")
    );
    assert!(rpc.pending_request(request).is_none());
}

#[test]
fn request_response_timeout_fires() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "request-timeout-player");
    rpc.register_rpc_handler(RpcDescriptor::command("chat.request_timeout"), |_| {
        thread::sleep(Duration::from_millis(50));
        Ok(b"late".to_vec())
    })
    .unwrap();

    let request = NetRequestId::new(191);
    let timed_out = rpc.invoke_rpc(
        RpcInvocationDescriptor::new(
            "chat.request_timeout",
            RpcDirection::ClientToServer,
            Vec::new(),
        )
        .with_source_session(session)
        .with_request(request)
        .with_timeout_ms(10),
        RpcPeerRole::Client,
    );

    assert_eq!(timed_out.status, RpcDispatchStatus::TimedOut);
    assert_eq!(timed_out.request, Some(request));
    assert_eq!(timed_out.response_payload, None);
    assert!(rpc.pending_request(request).is_none());
}
