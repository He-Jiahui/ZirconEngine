---
title: Runtime operation in-flight retention and maintenance index performance review
date: 2026-08-23
module: runtime operation service and direct dynamic App Editor adapters
priority: MVP-P0 runtime progress and basic editor background operation control
status: current_source_reviewed_m0_m1_applied_static_validated_dynamic_pending
reference_engine: Unreal Tasks completion ownership and TimerManager deadline heap
---

# Goal

Keep the current bounded owner-snapshot/worker-prepare/owner-apply direction, but close the
in-flight capacity leak before optimizing the scheduler. Then replace frame-wide deadline scans,
batch receiver scans, repeated JSON accounting, and tick-only progress with indexed, owned,
wake-driven operation control. No current-source timing claim is accepted until managed Windows
Cargo and product profiling can execute.

## Current-source review fingerprint

Repository HEAD `9fee3ea0435961a81c85aa2502e64f1f357345d7` was reviewed end to end:

| owner | files reviewed | lines | bytes | joined-content SHA256 |
| --- | ---: | ---: | ---: | --- |
| `zircon_runtime/src/operation/**` | 12/12 | 3,075 | 110,119 | `baa388629b4bb2f41bd351666432d955546ec3ee50cc10fb82d8b19cde23a5d1` |
| runtime dynamic, App runtime-library, Editor session adapters | 3/3 | 572 | 20,837 | `9106fff06322378ef12b4b92b80c4b8678dd4c1fb0d9c43d2ef2aab7aed65c20` |

Post-M1, the operation folder is 13 files, 3,215 lines, 114,811 bytes, joined-content SHA256
`11baf3f33b91d0b0cafbc7160fd0874a3454a1b940b4d35edc14116e178692fc`; the added file is the
blocking-worker behavior gate and the production delta includes M0 plus nine scale counters.

The 12-file set includes all production owners and all 27 inline/source/phase-index tests. The
three modified operation files already present in the shared worktree contain formatting-only
changes and are preserved. The 2026-07-19 performance record covered the old 7-file/447-line
synchronous service; the 2026-08-16 Optimize Runtime41 report covered 11 files before the indexed
phase-queue test module landed. Neither older fingerprint is used as current acceptance.

## Preserved improvements

- Admission is bounded at 1,024 tasks, 32 worker prepares, 4 MiB retained JSON, and eight owner
  applies per tick by default.
- Work is split into owner snapshot, scheduler prepare, and owner apply. Prepare/apply panics,
  worker-channel loss, cancel, deadline, result TTL, and transactional harvest are represented.
- Queued snapshot and ready-apply selection now use FIFO `VecDeque` indexes. The source contains an
  ignored release benchmark with probe model `T*(T+1)/2 -> T` per cycle; it has not executed here.
- Poll is a fixed-layout V2 out parameter and performs no JSON/result allocation.

## Structural findings

### P0: tombstone pressure can permanently leak all prepare permits

A cancelled or expired `Preparing` task deliberately retains `prepare_in_flight=true` until its
worker completion is observed. `evict_tombstones_until_admissible`, however, may remove any
`Cancelled` or `Expired` task at capacity without checking that lease. The later completion cannot
find the task, so `release_prepare_slot` never decrements `state.in_flight_prepares`; channel-loss
recovery also counts only tasks still present. Repeating this at the default limit can leak all 32
permits and leave every later operation queued forever.

M0 must make in-flight worker ownership non-evictable. Capacity pressure may reject a new request
until completion arrives; after completion releases the permit, the terminal metadata can be
evicted normally. A deterministic blocking-worker behavior test must cover cancel, capacity
pressure, completion, permit release, and later admission.

### P1: every runtime frame performs four task-table expiry scans

`RuntimeOperationService::tick` calls deadline expiry three times and terminal TTL expiry once.
Each helper traverses the whole `HashMap`, allocates an expired-handle `Vec` even when empty, then
looks up matching tasks again. With `T` retained tasks the static frame cost is four full table
passes plus up to two-phase lookups, independent of actual expiry count. Timer refresh separately
scans the table twice to select the earliest deadline/TTL and is called after multiple transitions.

The hard-cut target is a generation-safe min deadline/TTL index with lazy stale-entry rejection:
peek O(1), insert/pop O(log T), and process only `K` due entries. Tick keeps three semantic expiry
barriers around completion/apply/snapshot, but each barrier becomes an indexed due check rather
than a full scan. Source counters must report table size, heap entries/stale pops, due rows, scans,
and maintenance re-arms.

### P1: completion ownership is batch-channel scanning rather than one service completion port

Every dispatching tick creates a new synchronous channel. Draining one message locks the receiver
vector and scans from index zero; the outer loop repeats that scan for every completion. Replace it
with one bounded service completion port whose permit/lease is owned until its completion or
explicit channel-loss receipt. Completion arrival must request host work; it must not wait for an
unrelated runtime frame or force redraw.

### P1: JSON is serialized repeatedly for accounting and transport

App/Editor submit serializes the request, runtime decodes it to a DOM, then service serializes the
payload again only to count bytes. Worker completion serializes command and result for byte counts;
harvest serializes the result again for ABI output and the caller decodes it. Keep JSON only at the
compatibility boundary and establish typed/canonical owned bytes and accounting once.

### P1: count budgets do not bound owner-thread time

One tick may run eight arbitrary snapshots and eight arbitrary applies while holding the runtime
session/world owner. `RuntimeOperationContext::world_mut` also contradicts the snapshot comment.
Runtime41 must provide a read-only snapshot context, time/operation budgets, typed commit
disposition, and at least two real product consumers before this becomes an accepted control plane.

## Unreal source basis

- `Core/Public/Tasks/Task.h:617-627` states that cancellation does not erase task completion:
  waiting remains blocked until prerequisites and execution complete. The transferable invariant is
  that cancellation changes publication intent, not ownership of an in-flight completion lease.
- `Engine/Public/TimerManager.h:568-579` owns timer data in a sparse array and active deadlines in
  `ActiveTimerHeap` rather than finding the minimum by scanning all timers every frame.
- `Engine/Private/TimerManager.cpp:1212-1248` peeks `HeapTop`, stops when the earliest timer is not
  due, and pops only due or pending-removal entries. Zircon should copy this indexed deadline
  behavior, not Unreal's object model.
- `Core/Public/Async/TaskGraphInterfaces.h:215-218,309-373` makes execution thread and completion
  handles/waits explicit. Zircon still needs nonblocking session-owned completion wake and must not
  wait on the runtime owner thread.

## Milestones and gates

| milestone | work | gate |
| --- | --- | --- |
| M0 | Block eviction of terminal metadata while a worker prepare lease remains in flight. | cancelled/expired preparing task rejects pressure admission; completion releases exactly one permit; later admission succeeds |
| M1 | Add operation size/phase/deadline/completion counters and current-source stress harness. | operation count, retained bytes, scans, due rows, receiver probes, owner callback time visible |
| M2 | Replace deadline/TTL full scans with generation-safe min indexes. | idle expiry table probes `4*T/frame -> O(1)`; due work `O(K log T)`; lifecycle parity |
| M3 | Replace per-tick batch channels with one bounded owned completion port and wake receipt. | receiver probes scale with completions; no lost permit; reactive queue reaches terminal without unrelated input/redraw |
| M4 | Typed/canonical request, command, and result ownership plus read-only snapshot and timed apply budget. | repeated JSON count serializations removed; owner p95/p99 and effect disposition accepted |
| M5 | Shutdown fence, real navigation plus second consumer, 1/1K/100K dynamic qualification. | zero task/lease after session destroy; bounded RSS/queue; WPR/energy and product behavior evidence |

## Validation state

- Current 12/12 operation files and 3/3 direct adapters are statically reviewed. No module is moved
  to `review.md`.
- M0 is applied: pressure eviction now excludes `prepare_in_flight` terminal tasks. Static potential
  permit loss changes from up to the configured 32 in-flight prepares to zero through this eviction
  path; pressure admission rejects until worker completion releases the lease.
- M1 observability is applied. Five maintenance counters expose deadline/TTL scanned and expired
  rows plus earliest-deadline selection rows. Four completion counters expose receiver rows,
  receiver probes, completion rows, and lost rows. They have not produced current-source samples.
- Focused static contract
  `tools/tests/test_runtime_operation_inflight_retention_performance_contract.py` is 57 lines, 2,112
  bytes, SHA256 `1ab52f7fc60013cdcbf07e2b5a0b24e08334e7524dea51de647944aebed5ea96`.
  M0 ran RED `1/2` to GREEN `2/2`; M1 ran RED `2/3` to GREEN `3/3`. Rustfmt
  `+1.94.1 --edition 2021 --check` and scoped `git diff --check` pass.
- The Rust blocking-worker test covers cancel while preparing, capacity rejection, completion, and
  later admission. It has not executed because the managed Session cannot enter Cargo.
- Existing tests cover phase order, FIFO indexes, bounded admission, panic, TTL, cancellation,
  channel loss, and harvest. They do not cover in-flight tombstone eviction or maintenance
  complexity.
- Managed Cargo remains unavailable to this Session (`cargo_session_not_executable`); no Rust test,
  WPR, allocator, power, or current-source product result is claimed.
- RenderDoc is not applicable to this CPU/job slice unless later operation work changes the render
  product; no stale executable or capture is used.
