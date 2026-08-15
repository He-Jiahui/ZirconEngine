---
related_code:
  - zircon_editor/src/core/process.rs
  - zircon_editor/src/ui/host/export_process_support
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: implementation-evidence
status: routing_blocked_by_protected_plan_owner
created_at: 2026-08-15
---

# Editor process supervision/output protected-plan routing evidence (2026-08-15)

## Coordinator decision

The current review is complete in
`2026-08-15-editor-process-supervision-output-current-review.md`: 10/10 Rust files, 2,927 physical
lines, 16 inline tests and manifest
`871709381a80334aabe34b65ee347b21b8470c7fb14f06193c081f31e21aca2e`.

Performance01 may write only below `docs/plans/performance/01/**`. Main performance indexes and the
Editor/Runtime plans remain owner-protected. This file requests merges without editing those paths.
It deliberately strengthens existing PERF-MVP-080/091/639 instead of creating a duplicate task.

## Required owner merges

### Performance main plan

Strengthen three existing rows:

| Existing ID | Required merge | Acceptance addition |
|---|---|---|
| PERF-MVP-080 | Cargo's full stdout/stderr Vec then String is only one manifestation. `final_output_drain` aggregates all terminal remainder despite 64-KiB live reads, and capture first writes full bytes to temporary files | terminal remainder allocation bounded; result is locator/digest/tail; one canonical full write; 1-GiB fast-exit fixture has constant output working set |
| PERF-MVP-091 | wizard now has a 512-line O(1) deque tail and 16-KiB decoder, but bytes travel child -> temporary capture -> memory chunk -> durable artifact, with terminal full aggregation and per-line UI callbacks | preserve current line/tail limits; remove duplicate full writes and terminal aggregation; UI deltas obey count/byte/time and full output remains available by artifact/digest |
| PERF-MVP-639 | Play's process backend depends on a shared flaw: tree termination consumes the lease, Windows spawn scans every system thread to resume the primary thread, and Play/export have separate tree owners | one Runtime11 `ProcessSessionGeneration` and terminal receipt chain; direct primary-thread handle, retryable termination/reap/pipe/artifact ownership, no private threads/sleep polling/blocking Drop/taskkill steady path |

### Performance pending/review indexes

Update existing concise module rows rather than adding per-file rows:

- under `zircon_editor/src/core/**`, record `core/process.rs` current 1/1, 658 lines, four tests,
  static reviewed/dynamic pending;
- under `zircon_editor/src/ui/**`, record export process support/Cargo/wizard execution current 9/9,
  2,269 lines, 12 tests, static reviewed/dynamic pending;
- link the 10-file fingerprint and current report.

Keep both roots out of `review.md` until B1-B4, managed Cargo, failure/retry/tree/pipe tests,
1-GiB output and 100-process scale, F4 Play/export product runs, WPR/xperf, RSS and energy pass.

### Plan02 M1 and M4

M1 must define the shared `ProcessSupervisor`/ticket contracts inside the single TaskGraph: process
lane, platform resource handles, output/artifact policy, cancellation/deadline/escalation and typed
terminal receipts. Editor-private process workers are forbidden.

M4 must migrate Play and editor export consumers to this owner after Runtime11 lands, then delete
the duplicate controller/backend/export process authorities. No synchronous compatibility wrapper,
dual tree policy or blocking destructor survives.

### Runtime11 owner

Add one cross-product process service with this dependency order:

1. `ProcessSpec` and session/generation/receipt schema;
2. Windows direct create/Job attach/primary-thread resume and Unix group/session ownership;
3. shared wait/readiness, bounded output delta and canonical artifact writer;
4. retryable terminate -> reap -> pipe close -> artifact cleanup stages;
5. metrics for queue wait, spawn, output bytes/high-water, cancel/escalate/reap and resources;
6. old `taskkill`, Toolhelp thread discovery and process sleep-poll deletion.

Do not copy Unreal's private `FMonitoredProcess` thread or process-snapshot tree traversal. Reuse its
central process/pipe ownership lesson while applying Zircon's Runtime11 scheduling contract.

### Editor14 owner

Converge the existing job plan from nested `join` plus 25/100-ms sleeping loops to typed process
tickets. A process may live for hours without occupying a general CPU worker. Completion and output
deltas return through generation-checked job events. `ExportProcessChildGuard::Drop` becomes an
idempotent nonblocking cleanup submission and is deleted after all callers migrate.

### Editor15 owner

Merge PERF-MVP-080/091 into one output contract:

- one declared canonical stdout/stderr artifact per stage;
- streaming BLAKE3/byte counts and a 512-line bounded tail;
- bounded line decoding and UI delta backpressure;
- report stores locator/digest/count/tail, not complete output;
- no temporary full relay files, complete Vec/String, terminal full drain or duplicate fsync chain;
- every artifact/test root supplied from approved D/E/F project/build/validation paths.

### Editor04 owner

Consume the same process ticket in Play. Process, tree, stdout/stderr and Play snapshot ownership
remain live until terminal receipts. A termination/plugin cleanup failure projects an explicit
cleanup-pending state and remains retryable; it cannot erase the process owner or report false
`Playing`.

## Deletion and acceptance gate

The shared milestone remains open while any product path:

- calls `CreateToolhelp32Snapshot` to find the just-created primary thread;
- launches `taskkill` as its normal tree authority;
- consumes the only tree/process owner before termination and reap receipts;
- performs process wait/kill/spawn or output cleanup in `Drop`;
- sleeps a general CPU worker to poll a process;
- aggregates terminal output or returns complete output Vec/String;
- writes full output first to a temporary relay and then to its canonical artifact;
- writes process fixtures/artifacts/traces to C:;
- keeps separate Play and export process supervision implementations.

No commit or WeCom notification is due for this static routing record. Commit and quantified WeCom
notification occur only after the shared hard-cut milestone has accepted current-source dynamic
evidence.
