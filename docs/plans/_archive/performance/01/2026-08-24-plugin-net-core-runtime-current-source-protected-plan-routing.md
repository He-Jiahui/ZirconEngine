---
title: Plugin Net Core Runtime Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-plugin-net-core-runtime-current-source-performance-review.md
---

# Plugin Net Core Runtime Protected Plan Routing

## Review ledger status

`zircon_plugins/net/runtime` completed an E3 current-source static review over **49/49 Rust files**. Protected `docs/plans/performance/review.md` and `docs/plans/performance/pending.md` remain unchanged because no managed Cargo, current-source product trace, soak, WPR/ETW or power receipt exists. The 2026-07-30 report's core-runtime freeze is stale; broader Net feature ownership remains with Plugins10.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Two multi-thread Tokio runtimes plus one OS thread per manager/provider, while TCP/UDP remains serial `block_on` | Runtime08E, Runtime59 and Plugins10 | Define one World/session network instance and one certified I/O authority; forbid executor creation per manager/feature and gate optional receive parallelism by workload. |
| Synchronous `NetManager` methods wait up to 2 seconds; HTTP/WS can block longer; timeout does not cancel late side effects | Runtime08E, Runtime59, Runtime24 and Plugins10 | Hard-cut to generation-qualified async/ticket operations with deadline, cancel, terminal receipt and zero late publication. |
| One slow connect or non-writable TCP stream blocks every connection on the serial worker | Runtime08E and Runtime59 | Implement readiness-driven per-connection state and bounded dispatch/flush batches; prove independent-connection progress. |
| Diagnostics drains worker ingress with `usize::MAX` before a 256-event frame drain; main/WS queues are unbounded by bytes and age | Runtime08E, Runtime54 and Runtime03 | Add entry+byte+age+owner budgets, overflow policy and O(1) diagnostic snapshots; diagnostics must never consume work. |
| `NetConfig`/manifest budgets are not wired, `NetDriver` is empty, `TickFlush` is a no-op and reconnect policy is test-only | Runtime08E, Runtime55 and Plugins10 | Produce validated `EffectiveNetConfig`; make declared owners executable or remove their full stale surface in one hard cut. |
| UDP/TCP poll allocations, pre-admission payload copies, linear routes, per-request HTTP/TLS clients and full response collection | Runtime08E and Plugins10 HTTP/WebSocket milestones | Add retained buffers/slabs, admission-before-copy, compiled route index, configuration-generation client pools and streaming request/response limits. |
| Optional HTTP/WS managers and other features do not share the canonical session/connection authority | Plugins10, Runtime42 and Runtime08E | Attach feature providers to the same activation plan and network instance; add source/native/product parity receipts. |
| Existing diagnostics omit queue bytes/age/drop, worker queue/service wall, late completion, copies, allocations, wakeups and shutdown wall | Runtime03, Runtime08E and Editor25 | Establish instrumentation before implementation and bind every sample to BuildSet, target, hardware, session generation and workload. |

## Acceptance routing

The module may move from static-reviewed to accepted only after M0-M7 establish product/currentness truth, baseline instrumentation, non-blocking cancellable operations, unified I/O ownership, bounded dispatch/flush, dual-dimension backpressure, allocation/client/route convergence, feature-product integration and controlled dynamic evidence.

Required receipts include zero frame-thread synchronous network wait; executor/thread count independent of manager and feature count; queue memory bounded by configured bytes; per-frame packet/byte/wall gates; no cross-connection head-of-line stall; zero steady-state UDP no-data allocation; pooled HTTP/TLS clients; zero orphan publication after timeout/cancel/stale generation; and BuildSet-bound P50/P95/P99, RSS, context-switch/wakeup and energy results.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
