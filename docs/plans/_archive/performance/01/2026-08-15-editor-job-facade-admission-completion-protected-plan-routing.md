---
related_code:
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/notifications
  - zircon_editor/src/ui/retained_host/app/job_progress.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: implementation-evidence
status: routing_blocked_by_protected_plan_owner
created_at: 2026-08-15
---

# Editor job facade admission/completion protected-plan routing evidence (2026-08-15)

## Coordinator decision

The current review is complete in
`2026-08-15-editor-job-facade-admission-completion-current-architecture-review.md`: 47/47 Rust
files, 9,084 physical lines, 108 inline tests and raw manifest
`1bae4346aab7da598768715f7e7fc381795321cb4257b31de446e21f80325df1`.

Performance01 may write only below `docs/plans/performance/01/**`. The main performance indexes and
Editor/Runtime plans remain owner-protected. This file requests merges without editing those paths.
It strengthens existing PERF-MVP-017/018/020/627 instead of creating duplicate work.

## Required owner merges

### Performance main plan

Update the four existing rows to current-source truth:

| Existing ID | Required correction | Acceptance addition |
|---|---|---|
| PERF-MVP-017 | Primary-generation state and tests now exist, but production still calls `primary_snapshot()` every tick; source clone/format/DTO comparison remain | wire a retained generation cursor through status access; stable ticks have zero clone/format/invalidation |
| PERF-MVP-018 | Pending entry/byte/age admission is now bounded and progress latest-coalesces; accepted Started/terminal queue rows remain unbounded and downstream lossless | admission reserves lifecycle/result/presentation entries+bytes through terminal bus handoff; expose queue depth/bytes/age/high-water |
| PERF-MVP-020 | Finite category limits, 64-job lock-external dispatch and indexed terminal eviction are implemented; remaining issue is recursive worker completion -> observer -> promotion and duplicate Editor/Runtime scheduler authority | delete editor scheduler policy after Runtime11 lane migration; worker callbacks/promotion attempts/observer execution=0; shared immutable progress payload |
| PERF-MVP-627 | The one Runtime TaskGraph hard cut must absorb Editor category/priority/dependency/mutex scheduling, not merely its Rayon pools | one queue/worker/affinity/receipt truth across Runtime and Editor; no facade compatibility scheduler |

### Performance pending/review indexes

Replace the concise current module row for `zircon_editor/src/core/jobs/**` with:

- current 47/47 files, 9,084 lines, 108 tests;
- static review complete, dynamic acceptance pending;
- current fingerprint and review link;
- fixed facts: finite limits, 16,384-entry/64-MiB/five-minute pending admission, indexed
  fairness, 64-item lock-external dispatch, ordered terminal eviction, shared labels and generation
  API;
- remaining P0: duplicate scheduler authority, unbounded accepted lifecycle retention, synchronous
  worker observer/promotion chain and unused production generation fast path.

Keep the module out of `review.md` until managed current-source Cargo, deterministic scheduling and
retention counters, F0/F4 product runs, WPR/xperf, RSS and energy are GREEN. Do not mark the module
accepted from source-shape tests or the existing 1K wall-clock storm alone.

### Plan02 M1

Make M1's `TaskGraph` contract explicitly absorb Editor job scheduling:

1. one dependency node/ticket/terminal receipt type;
2. one global worker set plus named main/editor/render affinity executors;
3. priority and resource lanes that express Editor categories without a second queue;
4. atomic count+byte+age admission for payload, result and lossless lifecycle retention;
5. coalesced completion wake and deadline-bounded named-affinity observer delivery;
6. one always-on low-cost queue/worker/wake/retention diagnostic truth;
7. hard deletion of the Editor ready/dependency/mutex-tail/promotion scheduler after migration.

### Runtime11 owner

Define the shared TaskGraph API before implementation:

- `TaskSpec { priority, affinity, resource_lane, estimated_bytes, deadline, prerequisites }`;
- typed accepted/rejected/merged outcomes and generation-stamped terminal receipts;
- no foreign/user/UI callback on a general Runtime worker completion path;
- completion wakes a named-affinity executor without recursive scheduler calls;
- pending plus started-but-undelivered retention remains within one reservation;
- blocking wait rejects named main/editor/render callers; shutdown has typed deadline receipts;
- metrics expose queue depth/bytes/oldest age, start latency, execution, completion delivery,
  worker peak, park/wake/steal, callback time and rejected/coalesced work.

Use Unreal's single `FTaskGraphInterface`, explicit named thread/priority and trigger-on-completion
contracts as the primary local reference. Do not copy indefinite `WaitUntilTasksComplete` behavior.

### Editor14 owner

Reduce `EditorJobSystem` to a facade. Preserve typed domain categories, quota configuration,
cooperative cancellation, keyed latest-work admission, progress receipts and result tickets, but
remove its own records, ready buckets, dependency handles, mutex tails, promotion gate and
completion scheduling. Map categories to Runtime11 resource lanes through one immutable policy
table. Domain-specific serialization becomes a Runtime lane/key, not an Editor-owned dependency
chain.

Completion must enqueue one immutable receipt and return. Notification resynchronization and status
projection run on the named editor consumer with count+byte+deadline budgets. A completion storm
must cause one coalesced wake, not one promotion attempt per job.

### Editor02 owner

Join job event retention to the message-bus budget:

- reserve accepted Started/terminal edges before job acceptance;
- keep terminal lossless only inside the hard reservation;
- keep progress latest with a message byte/update-rate limit;
- drain by count+bytes+deadline and report remaining/oldest age;
- share one immutable event/progress payload between job state and delivery;
- preserve per-job lifecycle ordering and explicit backpressure semantics.

### EditorUI08 owner

Wire `primary_snapshot_if_changed` into production with a retained generation cursor. The unchanged
path must return before label/progress clone, task/detail formatting, status DTO construction,
workbench preparation or invalidation. Task-panel reads remain explicit bounded/paged snapshots.

## Deletion and acceptance gate

The shared milestone remains open while any product path:

- retains Editor-owned ready/dependency/mutex-tail/promotion scheduling beside Runtime TaskGraph;
- executes notification/UI observers from Runtime worker completion;
- performs one promotion-lock attempt per completion;
- accepts a job without reserving lossless lifecycle/result/presentation retention;
- stores lifecycle rows in a queue without count/byte/age bounds and high-water metrics;
- clones the progress message into both authoritative state and the queue;
- calls `primary_snapshot()` and formats status DTOs on every stable retained tick;
- permits named main/editor/render callers to use `JobTicket::wait`;
- writes job tests, traces or temporary roots to C:;
- treats source-shape tests or one wall-clock run as dynamic performance acceptance.

No commit or WeCom notification is due for this static routing record. Commit and quantified WeCom
notification occur only after the shared hard-cut milestone has accepted current-source dynamic
evidence.
