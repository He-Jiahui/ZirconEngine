---
title: Plugin Net RPC Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-plugin-net-rpc-current-source-algorithm-performance-review.md
---

# Plugin Net RPC Protected Plan Routing

## Review ledger status

RPC **20/20** Rust files completed E3 current-source static review. Shared changes in `manager/channel.rs` and `manager/session.rs` were preserved; this audit made no source change. Protected `review.md` and `pending.md` remain unchanged because Cargo, multi-process product, fault/scale, WPR and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Feature ignores NetManager and has no production transport/World caller | Runtime08E P1-15/M5 and Plugins10 M5 | Install one per-World/session RPC router consuming canonical authenticated connections; remove private production manager paths. |
| Token is discarded; challenge is static; role/session/NetSpeed are caller-controlled | Plugins10 NNET-P1-034/G20/G30 and Runtime08E M4-M5 | Bind principal, role, session generation and rate policy to authenticated connection context; reject replay and caller-forged authority. |
| Synchronous handler checks timeout after return and permits late side effects | Plugins10 G13/G14/G21 and Runtime08E M5 | Add bounded cancellable execution, absolute deadlines, generation-checked commit and explicit budgeted main-thread bridge. |
| Mutable string registries and validator callbacks run through one global mutex | Editor26 RPC authoring owner and Plugins10 M5/G21 | Compile immutable versioned stable-ID codecs/permissions/execution policy; use generation-qualified handler leases outside mutable locks. |
| Global heap/channel queues lack byte/age/per-owner fairness and transport accounting | Plugins10 G19/G21 and Runtime08E M5 | Add hierarchical items/bytes/age/rate/in-flight budgets, fair aging, packet-byte admission and overload receipts. |
| Request IDs overwrite globally and have no response/cancel/dedup terminal protocol | Plugins10 NNET-P1-035/G21 | Namespace correlation by connection generation and guarantee exactly one bounded terminal outcome across loss, duplicate and disconnect. |
| Close only marks sessions and retains quotas, work, requests and channel rows | Plugins10 G12/G16/G20 | Make teardown idempotent, cancel/quiesce generation-owned work and prove zero retained rows/tasks/callbacks. |
| Local exact-capacity/heap/batched-scan improvements are unexecuted and incomplete | Plugins10 current implementation record | Preserve compatible wins, fix duplicate-close amplification, then measure only inside the rebuilt product pipeline. |

## Acceptance routing

Implementation must start with the compiled RPC artifact and authenticated connection binding, followed by cancellable execution, correlation, channel/transport integration and lifecycle telemetry. Further HashMap, clone or heap tuning before these owners exist would optimize a disconnected model and cannot establish engine performance.

Dynamic acceptance requires separate server/client current-source processes plus malformed input, flood, loss, cancellation, disconnect, unload and soak. WPR/ETW results must bind tail latency, CPU, RSS, main-thread wait, wakeups, bytes and energy to the same BuildSet and workload. RenderDoc is relevant only when RPC load is correlated with a real rendered frame; it is not an RPC CPU profiler.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
