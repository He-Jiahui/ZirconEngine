use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Barrier,
};
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
    })
    .unwrap();
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
    rpc.register_schema_validator("schema://chat/fail.v1", |_| true)
        .unwrap();
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
fn rpc_feature_manager_returns_late_sync_handler_result_without_retryable_timeout() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "timeout-player");
    let effects = Arc::new(AtomicUsize::new(0));
    let handler_effects = effects.clone();
    rpc.register_rpc_handler(RpcDescriptor::command("chat.slow"), move |_| {
        handler_effects.fetch_add(1, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(50));
        Ok(b"late".to_vec())
    })
    .unwrap();

    let request = NetRequestId::new(91);
    let completed = rpc.invoke_rpc(
        RpcInvocationDescriptor::new("chat.slow", RpcDirection::ClientToServer, Vec::new())
            .with_source_session(session)
            .with_request(request)
            .with_timeout_ms(10),
        RpcPeerRole::Client,
    );

    assert_eq!(completed.status, RpcDispatchStatus::Accepted);
    assert_eq!(completed.response_payload, Some(b"late".to_vec()));
    assert_eq!(
        completed.diagnostic.as_deref(),
        Some("RPC handler completed after its deadline; synchronous execution was not cancelled")
    );
    assert_eq!(effects.load(Ordering::SeqCst), 1);
    assert!(rpc.pending_request(request).is_none());
}

#[test]
fn slow_request_response_returns_committed_result_after_deadline() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "request-timeout-player");
    rpc.register_rpc_handler(RpcDescriptor::command("chat.request_timeout"), |_| {
        thread::sleep(Duration::from_millis(50));
        Ok(b"late".to_vec())
    })
    .unwrap();

    let request = NetRequestId::new(191);
    let completed = rpc.invoke_rpc(
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

    assert_eq!(completed.status, RpcDispatchStatus::Accepted);
    assert_eq!(completed.request, Some(request));
    assert_eq!(completed.response_payload, Some(b"late".to_vec()));
    assert!(rpc.pending_request(request).is_none());
}

#[test]
fn schema_validator_can_reenter_manager_without_deadlocking() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "reentrant-validator-player");
    rpc.register_rpc(
        RpcDescriptor::command("chat.reentrant").with_payload_schema("schema://chat/reentrant.v1"),
    )
    .unwrap();
    let callback_manager = rpc.clone();
    rpc.register_schema_validator("schema://chat/reentrant.v1", move |_| {
        assert!(callback_manager.rpc_descriptor("chat.reentrant").is_some());
        callback_manager
            .register_rpc(RpcDescriptor::command("chat.other"))
            .unwrap();
        true
    })
    .unwrap();

    let report = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new("chat.reentrant", RpcDirection::ClientToServer, Vec::new())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(report.status, RpcDispatchStatus::Accepted);
}

#[test]
fn schema_validator_policy_replacement_invalidates_admission_snapshot() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "replaced-validator-player");
    let effects = Arc::new(AtomicUsize::new(0));
    let handler_effects = effects.clone();
    rpc.register_rpc_handler(
        RpcDescriptor::command("chat.replace").with_payload_schema("schema://chat/replace.v1"),
        move |_| {
            handler_effects.fetch_add(1, Ordering::SeqCst);
            Ok(Vec::new())
        },
    )
    .unwrap();
    let callback_manager = rpc.clone();
    rpc.register_schema_validator("schema://chat/replace.v1", move |_| {
        callback_manager
            .register_rpc(RpcDescriptor::command("chat.replace"))
            .unwrap();
        true
    })
    .unwrap();

    let report = rpc.invoke_rpc(
        RpcInvocationDescriptor::new("chat.replace", RpcDirection::ClientToServer, Vec::new())
            .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(report.status, RpcDispatchStatus::SessionUnavailable);
    assert_eq!(
        report.diagnostic.as_deref(),
        Some("RPC admission policy changed during validation")
    );
    assert_eq!(effects.load(Ordering::SeqCst), 0);
}

#[test]
fn schema_validator_session_change_is_rechecked_before_handler_admission() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "changed-session-player");
    rpc.register_rpc(
        RpcDescriptor::command("chat.close_during_validation")
            .with_payload_schema("schema://chat/close.v1"),
    )
    .unwrap();
    let callback_manager = rpc.clone();
    rpc.register_schema_validator("schema://chat/close.v1", move |_| {
        callback_manager.close_session(session).unwrap();
        true
    })
    .unwrap();

    let report = rpc.dispatch_rpc(
        RpcInvocationDescriptor::new(
            "chat.close_during_validation",
            RpcDirection::ClientToServer,
            Vec::new(),
        )
        .with_source_session(session),
        RpcPeerRole::Client,
    );
    assert_eq!(report.status, RpcDispatchStatus::SessionUnavailable);
    assert_eq!(
        report.diagnostic.as_deref(),
        Some("source session is closed")
    );
}

#[test]
fn handler_replacement_during_validation_invalidates_execution_snapshot() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "replaced-handler-player");
    let calls = Arc::new(AtomicUsize::new(0));
    let original_calls = calls.clone();
    let descriptor = RpcDescriptor::command("chat.handler_epoch")
        .with_payload_schema("schema://chat/handler_epoch.v1");
    rpc.register_rpc_handler(descriptor.clone(), move |_| {
        original_calls.fetch_add(1, Ordering::SeqCst);
        Ok(b"original".to_vec())
    })
    .unwrap();

    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let validator_entered = entered.clone();
    let validator_release = release.clone();
    rpc.register_schema_validator("schema://chat/handler_epoch.v1", move |_| {
        validator_entered.wait();
        validator_release.wait();
        true
    })
    .unwrap();

    let worker_manager = rpc.clone();
    let worker = thread::spawn(move || {
        worker_manager.invoke_rpc(
            RpcInvocationDescriptor::new(
                "chat.handler_epoch",
                RpcDirection::ClientToServer,
                Vec::new(),
            )
            .with_source_session(session),
            RpcPeerRole::Client,
        )
    });
    entered.wait();
    let replacement_calls = calls.clone();
    rpc.register_rpc_handler(descriptor, move |_| {
        replacement_calls.fetch_add(100, Ordering::SeqCst);
        Ok(b"replacement".to_vec())
    })
    .unwrap();
    release.wait();

    let report = worker.join().unwrap();
    assert_eq!(report.status, RpcDispatchStatus::SessionUnavailable);
    assert_eq!(
        report.diagnostic.as_deref(),
        Some("RPC admission policy changed during validation")
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn pending_handler_is_not_expired_while_sync_handler_is_running() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "active-handler-player");
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let handler_entered = entered.clone();
    let handler_release = release.clone();
    rpc.register_rpc_handler(RpcDescriptor::command("chat.active"), move |_| {
        handler_entered.wait();
        handler_release.wait();
        Ok(Vec::new())
    })
    .unwrap();

    let worker_manager = rpc.clone();
    let worker = thread::spawn(move || {
        worker_manager.invoke_rpc(
            RpcInvocationDescriptor::new("chat.active", RpcDirection::ClientToServer, Vec::new())
                .with_source_session(session)
                .with_request(NetRequestId::new(192))
                .with_timeout_ms(1),
            RpcPeerRole::Client,
        )
    });
    entered.wait();

    let expired = rpc.expire_pending_requests();
    let remains_pending = rpc.pending_request(NetRequestId::new(192)).is_some();

    release.wait();
    let report = worker.join().unwrap();
    assert!(expired.is_empty());
    assert!(remains_pending);
    assert_eq!(report.status, RpcDispatchStatus::Accepted);
    assert!(rpc.pending_request(NetRequestId::new(192)).is_none());
}

#[test]
fn pending_request_guard_cleans_up_after_handler_panics() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "panic-handler-player");
    rpc.register_rpc_handler(
        RpcDescriptor::command("chat.panic"),
        |_| -> Result<Vec<u8>, String> { panic!("intentional handler panic") },
    )
    .unwrap();
    let request = NetRequestId::new(193);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rpc.invoke_rpc(
            RpcInvocationDescriptor::new("chat.panic", RpcDirection::ClientToServer, Vec::new())
                .with_source_session(session)
                .with_request(request),
            RpcPeerRole::Client,
        )
    }));

    assert!(result.is_err());
    assert!(rpc.pending_request(request).is_none());
}
