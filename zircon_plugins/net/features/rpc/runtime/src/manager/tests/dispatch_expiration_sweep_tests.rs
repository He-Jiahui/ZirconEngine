use std::time::{Duration, Instant};

use zircon_runtime::core::framework::net::{
    NetRequestId, RpcDirection, RpcDispatchStatus, RpcInvocationDescriptor,
};

use super::{NetRpcRuntimeManager, PendingRpcRequest};

#[test]
fn expiration_sweep_removes_only_timed_out_requests() {
    let manager = NetRpcRuntimeManager::new();
    let expired_request = NetRequestId::new(501);
    let live_request = NetRequestId::new(502);
    let now = Instant::now();
    {
        let mut state = manager.state.lock().expect("net RPC state mutex poisoned");
        state.pending_requests.insert(
            expired_request,
            PendingRpcRequest {
                invocation: RpcInvocationDescriptor::new(
                    "expiration.expired",
                    RpcDirection::ServerToClient,
                    Vec::new(),
                )
                .with_request(expired_request)
                .with_timeout_ms(1),
                started_at: now - Duration::from_millis(10),
            },
        );
        state.pending_requests.insert(
            live_request,
            PendingRpcRequest {
                invocation: RpcInvocationDescriptor::new(
                    "expiration.live",
                    RpcDirection::ServerToClient,
                    Vec::new(),
                )
                .with_request(live_request)
                .with_timeout_ms(60_000),
                started_at: now,
            },
        );
    }

    let reports = manager.expire_pending_requests();

    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].status, RpcDispatchStatus::TimedOut);
    assert_eq!(reports[0].request, Some(expired_request));
    assert_eq!(
        reports[0].diagnostic.as_deref(),
        Some("pending RPC request timed out")
    );
    assert!(manager.pending_request(expired_request).is_none());
    assert!(manager.pending_request(live_request).is_some());
}
