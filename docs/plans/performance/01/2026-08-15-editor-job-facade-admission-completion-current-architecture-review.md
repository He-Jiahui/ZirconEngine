---
related_code:
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/notifications/progress
  - zircon_editor/src/core/notifications/service.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/status.rs
  - zircon_editor/src/ui/retained_host/app/job_progress.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphFwd.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/TaskGraph.cpp
  - dev/bevy/crates/bevy_tasks/src/usages.rs
tests:
  - 47 of 47 current Rust files reconciled and reviewed
  - 9084 physical lines and 108 inline tests
  - sorted path plus NUL plus raw bytes plus NUL SHA-256 1bae4346aab7da598768715f7e7fc381795321cb4257b31de446e21f80325df1
  - managed current-source Cargo and product WPR/xperf/energy remain blocked by the non-runnable editor baseline
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
created_at: 2026-08-15
---

# Editor job facade admission/completion current architecture review (2026-08-15)

## Scope freeze and method

This review freezes `zircon_editor/src/core/jobs/**` at **47/47 Rust files, 9,084 physical lines
and 108 inline tests**. The current raw-content manifest fingerprint is
`1bae4346aab7da598768715f7e7fc381795321cb4257b31de446e21f80325df1`. All files were read in
full, including the new folder-backed production owners and test scanners. Product reachability was
followed through notifications, retained status projection, autosave admission, Welcome project
probing, import/export jobs and the editor tick pump.

Seventeen tracked files and the new folder-backed owners contain foreign uncommitted work. This
pass reviewed that current workspace state and made no Rust edit. The current managed editor
baseline remains blocked at `tools/build-editor.ps1:130`: valid D/E/F roots are rejected before
Cargo because a PowerShell single-quoted separator contains two backslashes. WPR/xperf cannot
measure this source until the baseline produces a current executable. Dynamic latency, lock wait,
RSS, power and wake values are therefore `not_measured`; this root remains outside `review.md`.

## Architecture verdict

The current rewrite removes several real bottlenecks from the 2026-07-30 review:

- every category has a finite concurrency limit;
- pending admission has 16,384-entry, 64-MiB and five-minute defaults, plus byte-aware batch
  reservations and keyed latest-work merging;
- priority/category selection uses maintained ready buckets and at most 48 category probes per
  unsuccessful weighted-fair pass;
- promotion selects under the state mutex and calls Runtime `schedule_after` outside it, with a
  maximum batch of 64;
- terminal retention uses ordered evictable indexes and dependency reference counts instead of an
  interior `VecDeque` scan/remove;
- labels are shared as `Arc<str>`, progress is latest-coalesced, observer backlog collapses to one
  resynchronization after 1,024 entries, and a primary-generation API can skip stable reads.

Those changes fix the old unlimited-category, lock-held scheduler call, terminal-history scan and
repeated label allocation claims. They do **not** make this a unified TaskGraph facade yet.

The current completion chain remains:

`Runtime worker -> lifecycle mutex -> progress mutex -> event queue mutex -> result channel ->
CompletionGuard -> editor state mutex -> progress mutex -> observer queue mutex -> arbitrary
observer callback -> promotion mutex -> repeated Runtime schedule_after`

Editor still owns a second scheduler policy in front of Runtime: category quotas, weighted
priority, dependency readiness, mutex-group tails, records and promotion. Runtime11 owns another
dependency graph, pool queue and completion handle below it. The two layers cannot make one global
decision about affinity, worker saturation, I/O/process work, frame-critical work or queue age.
This is the structural P0: the editor job system must become a typed admission/presentation facade
over one Runtime11 TaskGraph, not remain a peer scheduler that forwards selected tasks to it.

## Current P0/P1 findings

### P0: accepted lifecycle retention is still unbounded

Pending admission capacity is released when a job starts. Each accepted job can then add a
`Started` edge and one terminal edge to `JobEventQueue.order`; these lifecycle rows have no
entry/byte/age reservation. The downstream editor message bus marks the same edges `Lossless`.
The 64-event/1-ms pump bounds frame work but not producer-side memory. A fast terminal storm can
therefore move work out of the bounded pending ledger and accumulate lossless lifecycle rows faster
than the retained tick consumes them. No queue depth, retained bytes, oldest age or high-water mark
exists to prove otherwise.

### P0: completion runs scheduling policy and observer callbacks on Runtime workers

`CompletionGuard::drop` synchronously calls `finish`, delivers the progress observer, notifies the
shutdown condition and calls `promote`. A slow observer directly holds the completing Runtime
worker. Fast jobs race back into the single promotion mutex; while a submitter or another worker
dispatches a 64-job batch, completing workers wait on that gate, and every completion later performs
another promotion attempt. The observer dispatch is reentrancy-aware and panic-contained, but it
has no execution-time budget and no named-affinity handoff.

Unreal's local source exposes one `FTaskGraphInterface::QueueTask` with explicit target/current
thread (`TaskGraphInterfaces.h:207-218`), task and worker priorities
(`TaskGraphFwd.h:55-77`), and completion triggering onto an explicit target thread
(`TaskGraphInterfaces.h:350-377`). Zircon should adopt that ownership model: worker completion
publishes a compact terminal receipt and one coalesced scheduling wake; a named editor-affinity
consumer performs presentation callbacks within a budget. It should not copy Unreal's indefinite
wait paths.

### P0: event capacity is not part of admission

The admission ledger charges only pending payload estimates. It does not reserve the two lossless
lifecycle edges, error width, progress-message width, result slot or observer receipt before
acceptance. A global 64-MiB pending estimate is therefore not a process memory bound. Admission
must reserve the complete retained lifecycle envelope and release it only after the terminal edge
has crossed the bounded main-thread/message-bus handoff.

### P1: stable status projection still does per-frame owned work

`primary_snapshot_if_changed` has a tested atomic stable-generation fast path, but production does
not call it. `status.rs` still calls `primary_snapshot`; `job_progress.rs` clones label/progress,
formats task/detail strings and compares the resulting DTO every retained tick. Equality avoids a
refresh, not the source clone and projection work. PERF-MVP-017 is therefore partially implemented,
not accepted.

### P1: progress publication retains two owned messages and crosses three locks

`report_progress` first materializes a `String`. `JobEventSink::emit` holds lifecycle state while
`ProgressState` clones that message into the active snapshot, then moves the original into the
event queue. Latest coalescing bounds row count per JobId but not allocation frequency, duplicate
live bytes or contention. The target is one immutable progress payload/generation shared by
authoritative state and presentation receipts, with message byte and update-rate budgets.

### P1: blocking and shutdown boundaries remain weak

`JobTicket::wait` has no main/named-thread affinity guard. `EditorJobSystem::join` synchronously
waits for borrowing work. Shutdown drains pending jobs through an allocated ID vector and O(N log N)
index removal, then runs cancellation callbacks serially on the caller. These APIs can remain for
worker/tool shutdown only, but product main-thread use must be structurally rejected and shutdown
must use bounded cancellation pages plus a deadline receipt.

### Verified non-findings

- Reservation `JobId` order still represents admission order even when reservations commit out of
  order: the allocation-time entries remain in the ledger until commit/release and preserve the
  original `admitted_at` ordering. No speculative fix is required.
- The retained primary lookup may cross terminal rows, but terminal emission is immediately
  followed by completion removal in the current job path; this is not the leading bottleneck.
- Message-bus handlers do not run under the bus mutex. Pump publication can still lock multiple
  bounded inboxes, but arbitrary subscriber callbacks are not part of this completion chain.

## Per-file reconciliation

| Current file | Review result |
|---|---|
| `admission.rs` | Compact typed keyed/batch admission DTOs; retain in the facade. |
| `cancellation_token.rs` | O(1) cooperative atomic token; lacks deadline/escalation by design. |
| `category.rs` | Enum-owned inventories prevent duplicate category lists; no hot allocation. |
| `context.rs` | Every progress call owns a message before coalescing or byte/rate admission. |
| `error.rs` | Typed errors share sources; owned text is confined to error/report paths. |
| `event_sink.rs` | Serial lifecycle -> progress -> queue lock chain and duplicate progress text. |
| `event.rs` | Shared label is fixed; progress/failure still own unbounded message strings. |
| `id.rs` | Scalar identity only. |
| `job.rs` | Typed work trait only; suitable facade boundary. |
| `limits.rs` | All categories finite; pending limits are real but do not include lifecycle retention. |
| `mod.rs` | Module/export wiring only. |
| `mutex_group.rs` | Validated owner string; construction-time cost, not a frame bottleneck. |
| `progress.rs` | Generation fast path exists, but production bypasses it; one global mutex remains. |
| `progress/primary_generation_tests.rs` | Good lock-free stable-read and generation contracts; product wiring is missing. |
| `pump.rs` | 64/1-ms consumption and latest progress are correct; lifecycle queue is unbounded/unmeasured. |
| `quota_settings.rs` | Four user quotas are restart-scoped; shared Runtime budget is not represented. |
| `shutdown.rs` | Compact report DTO; result projection clones only at explicit shutdown. |
| `spec.rs` | Shared label fixed; repeated `after` remains O(D^2) for dense construction. |
| `system/admission_ledger.rs` | Indexed counts/bytes/category/age; age order is valid, lifecycle capacity absent. |
| `system/admission_reservation.rs` | Rollback-safe capacity claim; commit still installs editor-owned scheduler records. |
| `system/construction.rs` | One injected Runtime scheduler, but Editor retains a second state/promotion authority. |
| `system/lifecycle.rs` | Cancellation/shutdown callbacks are synchronous; state -> progress lock nesting remains. |
| `system/mod.rs` | Clean folder-backed wiring. |
| `system/pending_task.rs` | Panic containment and latest payload replacement are sound; per-job boxes/channels remain. |
| `system/pending.rs` | 48-probe indexed fairness is bounded; drain is O(N log N) plus an ID vector. |
| `system/pending/tests/admission.rs` | Category counters/bytes/age stay coherent through merge/remove/drain. |
| `system/pending/tests/fairness.rs` | Weighted fairness is covered; no cross-Runtime lane saturation proof. |
| `system/pending/tests/mod.rs` | Test-only replacement stub and module wiring. |
| `system/progress_observer.rs` | 1,024-entry collapse and panic recovery are good; callbacks are synchronous/unbudgeted. |
| `system/scheduling.rs` | State lock is outside Runtime schedule; completion-to-promotion contention remains. |
| `system/state.rs` | Indexed terminal eviction/dependency pins fix prior scans; duplicate dependency assembly remains unused. |
| `system/submission.rs` | Admission is atomic; progress/observer registration nests under editor state ownership. |
| `test_support.rs` | Tests share a Runtime scheduler and avoid private production pools. |
| `tests/admission_scaling_contract.rs` | Folder-backed test wiring. |
| `tests/admission_scaling_contract/indexed.rs` | 1K/10K probe scaling is useful but not retained-memory/worker-contention evidence. |
| `tests/admission_scaling_contract/keyed.rs` | Merge/cancellation/age semantics covered. |
| `tests/admission_scaling_contract/reservation.rs` | Entry/byte/reservation rollback covered; event capacity is not. |
| `tests/admission_scaling_contract/support.rs` | Test jobs only; one busy-yield cancellation probe is validation-only. |
| `tests/background_storm_contract.rs` | Records 1K-job wall samples; vectors and sleep cadence make it diagnostic, not an SLA benchmark. |
| `tests/mod.rs` | Test wiring only. |
| `tests/progress_contract.rs` | Lifecycle visibility/cancel/shutdown covered; polling helpers busy-yield in tests only. |
| `tests/pump_contract.rs` | Pump order/budgets/coalescing covered; no queue entries/bytes/age high-water assertion. |
| `tests/quota_settings_contract.rs` | Quota persistence covered; temporary roots use `std::env::temp_dir()` and may write C:. |
| `tests/scheduling_contract.rs` | Broad semantics; sleeps/yields are test-only and no completion-gate contention counter exists. |
| `tests/thread_ownership_contract.rs` | Production bare-thread guard is valuable; filesystem scan is validation-only. |
| `tests/thread_ownership_contract/scanner.rs` | Large custom lexer is test-only; maintainability cost, not product runtime cost. |
| `ticket.rs` | `try_take` is short-lock polling; public unguarded `wait` is an affinity hazard. |

## Required hard-cut design

The target dependency chain is:

1. Runtime11 owns one process-wide `TaskGraph` worker set, named-affinity executors, dependency
   nodes, priority/resource lanes, queue admission and terminal receipts.
2. Editor14 reduces `EditorJobSystem` to typed request validation, domain quota requests,
   cancellation handles, result tickets and bounded presentation receipts. It owns no dependency
   graph, mutex tail, promotion loop or scheduled-handle history.
3. Submission reserves pending payload plus lifecycle/result/presentation bytes atomically from the
   Runtime lane. Rejection happens before channels, boxes or heavy payload materialization.
4. Completion publishes one immutable terminal receipt and one coalesced named-editor wake. Runtime
   workers never execute notification/UI observers or recursively run editor scheduling policy.
5. Editor02 drains lifecycle/progress receipts with count+bytes+deadline pages. Accepted terminal
   receipts are lossless within a hard reservation; progress is latest and byte/rate bounded.
6. EditorUI08 consumes `primary_snapshot_if_changed` through a retained generation cursor, then
   deletes unconditional per-frame primary clone/format/projection.
7. After all consumers migrate, delete the editor ready/dependency/mutex-tail/promotion scheduler
   and any compatibility forwarding path in the same milestone.

## Measurement and acceptance

- Scheduling: 1/1K/100K tasks, dependencies 0/1/10K, workers 1/2/N, all priorities/categories.
  Record one global queued/running/completed truth, lane depth/bytes/oldest age, enqueue and start
  p50/p95, steal/park/wake, completion-to-promotion calls, promotion-lock wait/hold and worker peak.
  Editor-private scheduler state and worker completion callbacks must be zero.
- Retention: events/job 0/2/1K, labels/errors/progress 0/4KiB/1MiB, consumer stalls 0/60s. Record
  reserved/live/high-water entries and bytes, oldest age, coalesced/rejected, allocation bytes,
  accepted terminal loss and downstream inbox pressure. Memory must remain within declared bounds.
- Stable UI: active jobs 0/1/1K/100K, stable ticks 1/100K and progress 0/1K Hz. Stable generation
  must produce zero snapshot/message clone, formatting and presentation invalidation.
- Shutdown: pending/running 0/1/100K, cooperative/non-cooperative/panic tasks and deadlines. Main
  thread waits must be zero; cancellation pages and terminal receipts must finish or return a typed
  timeout without blocking `Drop`.
- Run managed current-source focused jobs tests, Runtime11 TaskGraph tests and F0/F4 import,
  autosave, Welcome, export and thumbnail storms. Capture WPR/xperf CPU sampling, context switches,
  ready-thread/wake counts, RSS and energy on approved D/E/F paths. Require three repeatable runs
  before numerical acceptance.
- RenderDoc is not a CPU scheduler profiler. It becomes relevant only when the runnable F4 product
  trace correlates job stalls with viewport GPU markers; no GPU conclusion is claimed here.

## Reference boundary

Unreal is the primary structural reference, not a numerical oracle. Its local TaskGraph declares
named threads, worker/task priority bands, one queue interface, completion triggering and trace/stat
identity (`TaskGraphFwd.h:55-77`, `TaskGraphInterfaces.h:47-52,204-218,309-377`). Zircon should copy
the ownership clarity and explicit affinity while keeping bounded queues, deadlines and typed
receipts that Unreal's older blocking APIs do not guarantee. Bevy's compute/async-compute/I/O pool
separation is secondary evidence for resource classes, not authority for creating more pools.

## Static gates

- Current inventory: 47/47 files, 9,084 physical lines, 108 inline tests.
- Raw manifest fingerprint:
  `1bae4346aab7da598768715f7e7fc381795321cb4257b31de446e21f80325df1`.
- Current source confirms fixed finite category limits, indexed admission, 64-dispatch promotion,
  ordered terminal eviction, shared labels and the unused production generation fast path.
- `git diff --check -- zircon_editor/src/core/jobs` passed. Current `rustfmt --edition 2024
  --check` did not: `cancellation_token.rs`, `tests/progress_contract.rs` and
  `tests/pump_contract.rs` have formatting-only diffs in the foreign workspace rewrite. They were
  not rewritten by this review.
- No current-source managed Cargo, F0/F4 executable, WPR/xperf/energy trace or independent dynamic
  acceptance exists. The module remains pending.
