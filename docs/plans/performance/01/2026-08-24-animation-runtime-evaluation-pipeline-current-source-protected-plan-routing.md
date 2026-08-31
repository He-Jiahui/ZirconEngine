---
title: Animation Runtime Evaluation Pipeline Current-Source Protected Plan Routing
date: 2026-08-24
status: routing_record_only
source_review: docs/plans/performance/01/2026-08-24-animation-runtime-evaluation-pipeline-current-source-algorithm-performance-review.md
---

# Animation Runtime Evaluation Pipeline Current-Source Protected Plan Routing

## Review ledger status

The evaluation and channel-sampling production scope completed current-worktree static review and the scoped M0: **68/68 Rust files**, post-M0 fingerprint `efe888d3d708632b7629f7e4e3ba4ad5779bc8d17e62ff71e777672027ca720b`. Legacy leaf-name target resolution changed from O(T * B) comparisons to O(T * log B) ordered-map lookup and duplicate stored bone-name arrays changed from one to zero. Protected ledgers remain unchanged because managed Rust and current-source dynamic evidence are unavailable.

## Canonical owner notifications

| Finding | Canonical plan owner | Required update |
|---|---|---|
| Legacy leaf-name target compilation rescanned all bones and duplicated names | Plugins13 + Runtime08c | M0 implemented; retain unresolved/ambiguous contract coverage and dynamic compile-scale measurement. |
| Mutable World ownership encloses synchronous worker waits | Plugins13 + Runtime08c + Runtime59 + Runtime60 | Split extract/task/commit and model completion as a scheduler dependency. |
| Graph DAG evaluation is recursive and path-count based | Plugins13 + Runtime08c | Lower one bounded non-recursive program and execute each node once. |
| Worker shards duplicate resource caches/subscriptions | Plugins13 + Runtime04 + Runtime64 | Publish one generation-qualified compiled artifact cache with byte budgets. |
| Named pose vectors are rebuilt/copied through presentation, physics and scene commit | Plugins13 + Runtime08c + Runtime08a + Runtime09b | Use dense versioned pose handles and materialize names only for inspection. |
| Stable scans allocate and republish unchanged state | Runtime05 + Runtime60 + Editor69 | Add change generations and zero-work stable-frame contracts. |
| No animation relevance/LOD/time budget exists | Plugins13 + Runtime65 + Editor69 | Add measured budget, skip/interpolate/reduced-work and target-profile policy. |

## Acceptance routing

Implementation order is compile-time lookup M0 -> one instance/artifact generation -> topological program -> extract/task/commit -> dense pose publication -> relevance/budget -> dynamic qualification. A local lookup improvement cannot close this scope.

Dynamic acceptance records source/build/project identity, scale matrix, scan/queue/update/evaluate/commit p50/p95/p99, World lock/worker wait, allocations/bytes, compiled/cache bytes, duplicate compiles, skipped/reduced work, pose copies, transform writes, CPU, wakeups, RSS and power. No Git milestone commit or quantified WeCom message is warranted by this static routing record.
