---
title: Sound Dynamic Events ABI Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-sound-dynamic-events-abi-current-source-algorithm-performance-review.md
---

# Sound Dynamic Events ABI Current-Source Protected Plan Routing

## Review ledger status

Sound dynamic-event dispatch/executor/ABI **22/22 Rust files** completed E3 current-worktree static review at `2a1299f8bf8e5a3012860ff07a6fcf528e4721d8`; fingerprint `dbeb352940c144e0233b64b601125b11cb73fa57686ee19953c454d72ad64b93`. All files pass standalone rustfmt and scoped diff check; two shared import-order edits are preserved. Protected ledgers remain unchanged because Cargo, product event flow, native plugin unload, ETW and power evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| `dynamic_events_enabled` is stored but never enforced | Plugins11 + Runtime08b + Editor17 | Gate all admission/execution by applied capability/config generation and expose truthful status. |
| Pending queue, payload and event-handler fan-out are unbounded | Plugins11 + Runtime59 | Add quotas/caps, shared payload storage and explicit coalesce/drop/defer/fail policy. |
| All deliveries and executor map are cloned before serial caller-thread callbacks | Runtime59 + Runtime58 | Compile slot registry and run bounded affinity-aware tasks; audio thread receives commands only. |
| Raw ABI callback has no native module lease or unload quiescence | Runtime58 + Plugins11 | Add plugin generation/lease, draining, in-flight acknowledgement and stale callback rejection. |
| Failed/missing handlers consume events without retry/dead-letter semantics | Runtime58 + Runtime03 | Add delivery IDs, terminal policy and bounded diagnostic/dead-letter receipts. |
| Catalog/handler churn rebuilds global structures under Sound mutex | Plugins11 + Runtime58 | Prepare immutable registry generations off-lock and atomically publish them. |
| Existing benchmark covers only handler matching/indexing | Runtime03 + Editor25 | Add full pipeline burst/steady/slow/failure/unload/copy/queue-age measurements. |

## Acceptance routing

Implementation order is capability/config truth -> compiled registry -> bounded admission -> task execution -> plugin lifecycle -> delivery semantics -> dynamic qualification. Do not close this scope by optimizing handler lookup while payload fan-out and callback lifetime remain unbounded.

Dynamic acceptance records exact source/build/config/plugin/workload identity, event/handler/payload scales, submit/queue/dispatch/callback P50/P95/P99/max, throughput, queue depth/age, copies/bytes, coalesced/dropped/deferred/failed counts, CPU, main/audio stalls, RSS, wakeups, unload drain time and power.

No Git milestone commit or quantified WeCom message is warranted by this static routing record.
