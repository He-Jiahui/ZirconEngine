use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use zircon_runtime::core::framework::net::{
    NetConnectionId, NetControlMessage, NetError, NetRequestId, NetSessionHandshakeState,
    RpcDescriptor, RpcDirection, RpcDispatchStatus, RpcInvocationDescriptor, RpcPeerRole,
};

use super::{NetRpcRuntimeManager, NetRpcRuntimeState};

fn poison_state(state: &Mutex<NetRpcRuntimeState>) {
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let mut state = state.lock().unwrap();
        // Simulate a panic after one field of a multi-field login update commits.
        for session in state.sessions.values_mut() {
            session.handshake_state = NetSessionHandshakeState::Joined;
        }
        panic!("partial RPC protocol update");
    }))
    .is_err());
}

fn poisoned() -> NetError {
    NetError::SharedStatePoisoned {
        resource: "net.rpc_state".into(),
    }
}

fn invocation() -> RpcInvocationDescriptor {
    RpcInvocationDescriptor::new("poison.test", RpcDirection::ServerToClient, Vec::new())
}

#[test]
fn poisoned_protocol_state_fails_closed_across_clones_and_operations() {
    let manager = NetRpcRuntimeManager::new();
    let clone = manager.clone();
    let connection = NetConnectionId::new(1);
    let session = manager.begin_handshake_for_connection(connection).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let handler_calls = Arc::clone(&calls);
    manager
        .register_rpc_handler(RpcDescriptor::client_rpc("poison.test"), move |_| {
            handler_calls.fetch_add(1, Ordering::SeqCst);
            Ok(Vec::new())
        })
        .unwrap();
    assert_eq!(
        manager.enqueue_rpc(invocation(), RpcPeerRole::Server).status,
        RpcDispatchStatus::Queued
    );
    manager.enqueue_channel_message(0, 0, vec![1]).unwrap();
    poison_state(&manager.state);

    assert_eq!(clone.begin_handshake(), Err(poisoned()));
    assert_eq!(manager.handshake_state(session), Err(poisoned()));
    assert_eq!(manager.session_info(session), Err(poisoned()));
    assert_eq!(manager.close_session(session), Err(poisoned()));
    assert_eq!(
        manager.process_control_message(session, NetControlMessage::Join),
        Err(poisoned())
    );
    assert_eq!(
        manager.register_rpc(RpcDescriptor::client_rpc("new")),
        Err(poisoned())
    );
    assert_eq!(manager.register_schema_validator("new", |_| true), Err(poisoned()));
    assert_eq!(manager.enqueue_channel_message(0, 0, vec![2]), Err(poisoned()));
    for report in [
        manager.dispatch_rpc(invocation(), RpcPeerRole::Server),
        manager.invoke_rpc(invocation(), RpcPeerRole::Server),
        manager.enqueue_rpc(invocation(), RpcPeerRole::Server),
    ] {
        assert_eq!(report.status, RpcDispatchStatus::SessionUnavailable);
        assert!(report.diagnostic.unwrap().contains("net.rpc_state"));
    }
    assert!(manager.drain_rpc_queue(usize::MAX).is_empty());
    assert!(manager.drain_channel_messages(0, usize::MAX).is_empty());
    assert!(manager.expire_pending_requests().is_empty());
    assert!(manager.close_connection_sessions(connection).is_empty());
    assert!(manager.rpc_descriptor("poison.test").is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    // Replacement is explicit and cannot accidentally reopen the poisoned clone.
    assert!(NetRpcRuntimeManager::new().begin_handshake().is_ok());
    assert_eq!(clone.begin_handshake(), Err(poisoned()));
}

#[test]
fn poison_during_schema_callback_rejects_the_second_admission_phase() {
    let manager = NetRpcRuntimeManager::new();
    let calls = Arc::new(AtomicUsize::new(0));
    let handler_calls = Arc::clone(&calls);
    manager
        .register_rpc_handler(
            RpcDescriptor::client_rpc("poison.test").with_payload_schema("schema://poison"),
            move |_| {
                handler_calls.fetch_add(1, Ordering::SeqCst);
                Ok(Vec::new())
            },
        )
        .unwrap();
    let state = Arc::downgrade(&manager.state);
    manager
        .register_schema_validator("schema://poison", move |_| {
            poison_state(&state.upgrade().unwrap());
            true
        })
        .unwrap();

    let report = manager.invoke_rpc(invocation(), RpcPeerRole::Server);
    assert_eq!(report.status, RpcDispatchStatus::SessionUnavailable);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn pending_cleanup_does_not_panic_during_handler_unwind_after_poison() {
    let manager = NetRpcRuntimeManager::new();
    let state = Arc::downgrade(&manager.state);
    manager
        .register_rpc_handler(RpcDescriptor::client_rpc("poison.test"), move |_| {
            poison_state(&state.upgrade().unwrap());
            panic!("handler failed after another operation poisoned shared state");
        })
        .unwrap();

    let outcome = catch_unwind(AssertUnwindSafe(|| {
        manager.invoke_rpc(
            invocation().with_request(NetRequestId::new(1)),
            RpcPeerRole::Server,
        )
    }));
    assert!(outcome.is_err());
    assert_eq!(manager.begin_handshake(), Err(poisoned()));
}
