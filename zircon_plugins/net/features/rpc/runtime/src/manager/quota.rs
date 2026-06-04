use std::time::Instant;

use zircon_runtime::core::framework::net::{NetSessionId, RpcInvocationDescriptor};

use super::{
    state::{NetRpcRuntimeState, NetSpeedWindow, RpcQuotaKey, RpcQuotaWindow},
    NetRpcRuntimeManager, RPC_QUOTA_WINDOW,
};

impl NetRpcRuntimeManager {
    pub(in crate::manager) fn record_quota_call(
        state: &mut NetRpcRuntimeState,
        invocation: &RpcInvocationDescriptor,
        max_calls: u32,
    ) -> bool {
        let now = Instant::now();
        let key = RpcQuotaKey {
            rpc_id: invocation.rpc_id.clone(),
            source_session: invocation.source_session,
        };
        let window = state.quota_windows.entry(key).or_insert(RpcQuotaWindow {
            started_at: now,
            calls: 0,
        });
        if now.duration_since(window.started_at) >= RPC_QUOTA_WINDOW {
            window.started_at = now;
            window.calls = 0;
        }
        if window.calls >= max_calls {
            return false;
        }
        window.calls += 1;
        true
    }

    pub(in crate::manager) fn record_netspeed_bytes(
        state: &mut NetRpcRuntimeState,
        session: NetSessionId,
        bytes_per_second: u32,
        payload_bytes: usize,
    ) -> bool {
        let max_bytes = bytes_per_second as usize;
        if payload_bytes > max_bytes {
            return false;
        }

        let now = Instant::now();
        let window = state
            .netspeed_windows
            .entry(session)
            .or_insert(NetSpeedWindow {
                started_at: now,
                bytes: 0,
            });
        if now.duration_since(window.started_at) >= RPC_QUOTA_WINDOW {
            window.started_at = now;
            window.bytes = 0;
        }
        if window.bytes.saturating_add(payload_bytes) > max_bytes {
            return false;
        }
        window.bytes += payload_bytes;
        true
    }
}
