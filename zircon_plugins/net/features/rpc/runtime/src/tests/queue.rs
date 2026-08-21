use std::thread;
use std::time::Duration;

use zircon_runtime::core::framework::net::{
    NetRequestId, RpcDescriptor, RpcDirection, RpcDispatchStatus, RpcInvocationDescriptor,
    RpcPeerRole,
};

use super::support::complete_joined_session;
use crate::{NetRpcRuntimeManager, net_rpc_runtime_manager};

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
fn rpc_priority_queue_preserves_fifo_order_for_equal_priorities() {
    let rpc = net_rpc_runtime_manager();
    let session = complete_joined_session(&rpc, "fifo-player");
    rpc.register_rpc_handler(RpcDescriptor::command("chat.fifo"), |invocation| {
        Ok(invocation.payload.clone())
    })
    .unwrap();

    for request in 1..=3 {
        assert_eq!(
            rpc.enqueue_rpc(
                RpcInvocationDescriptor::new(
                    "chat.fifo",
                    RpcDirection::ClientToServer,
                    vec![request as u8],
                )
                .with_source_session(session)
                .with_request(NetRequestId::new(request))
                .with_priority(5),
                RpcPeerRole::Client,
            )
            .status,
            RpcDispatchStatus::Queued
        );
    }

    assert_eq!(
        rpc.drain_rpc_queue(3)
            .into_iter()
            .map(|report| report.request.unwrap().raw())
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
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

#[test]
fn rpc_feature_manager_marks_slow_queued_handler_timed_out_without_response_payload() {
    let rpc = NetRpcRuntimeManager::with_max_queue_depth(4);
    let session = complete_joined_session(&rpc, "queued-timeout-player");
    rpc.register_rpc_handler(RpcDescriptor::command("chat.slow_queue"), |_| {
        thread::sleep(Duration::from_millis(50));
        Ok(b"late".to_vec())
    })
    .unwrap();

    let request = NetRequestId::new(92);
    let queued = rpc.enqueue_rpc(
        RpcInvocationDescriptor::new("chat.slow_queue", RpcDirection::ClientToServer, Vec::new())
            .with_source_session(session)
            .with_request(request)
            .with_timeout_ms(10),
        RpcPeerRole::Client,
    );

    assert_eq!(queued.status, RpcDispatchStatus::Queued);
    let drained = rpc.drain_rpc_queue(4);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].status, RpcDispatchStatus::TimedOut);
    assert_eq!(drained[0].response_payload, None);
    assert_eq!(
        drained[0].diagnostic.as_deref(),
        Some("RPC handler exceeded invocation timeout")
    );
    assert!(rpc.pending_request(request).is_none());
}
