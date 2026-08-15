---
related_code:
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/notifications/service.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access/status.rs
  - zircon_editor/src/ui/retained_host/app/job_progress.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
plan_sources:
  - docs/plans/performance/01/2026-08-15-editor-job-facade-admission-completion-current-architecture-review.md
  - docs/plans/performance/01/2026-08-15-editor-job-facade-admission-completion-protected-plan-routing.md
  - docs/plans/performance/02/2026-08-15-runtime-taskgraph-current-architecture-review.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: implementation-evidence
status: current_hash_revalidated_dynamic_blocked
created_at: 2026-08-16
---

# Editor job facade current-hash revalidation (2026-08-16)

## Decision

The canonical 2026-08-15 architecture review remains current. This pass independently read all
**47/47 Rust files, 9,084 physical lines and 108 inline/module tests** under
`zircon_editor/src/core/jobs/**`. The raw-content fingerprint is unchanged:

`1bae4346aab7da598768715f7e7fc381795321cb4257b31de446e21f80325df1`

Production reachability was rechecked through notification progress observation, retained status
projection, the main tick event pump and Runtime `JobScheduler::schedule_after`. No new performance
task or replacement architecture is needed. The existing routing to PERF-MVP-017/018/020/627 and
Runtime11 remains authoritative.

## Revalidated facts

- finite per-category defaults, 16,384-entry/64-MiB/five-minute pending admission, keyed merge,
  48-probe weighted selection, 64-item lock-external promotion, indexed terminal eviction, shared
  labels and the primary-generation fast path are current fixes and must not be described as open;
- admission still releases its accounting when work enters Runtime, so it does not bound combined
  Runtime in-flight payload, result, accepted lifecycle event, observer or downstream message-bus
  bytes;
- `CompletionGuard` still invokes notification progress observers and the next promotion attempt on
  the completing Runtime worker. The observer is real product work: it allocates notification IDs,
  mutates the progress notification center and may refill from progress snapshots;
- the 64-event/1-ms pump does not bound the following synchronous observer-delivery loop, and
  accepted Started/terminal lifecycle rows remain lossless without an entry/byte/age reservation;
- production status projection still calls `primary_snapshot()` every retained tick instead of
  `primary_snapshot_if_changed`, then clones/formats a status DTO before equality suppresses only
  the final refresh;
- `shutdown(deadline)` drains and cancels all pending jobs synchronously before entering its
  deadline-governed wait. At the default 16,384 pending entries, cancellation/event/progress work can
  consume unbounded caller time relative to the supplied deadline;
- `JobTicket::wait` and synchronous `EditorJobSystem::join` remain public without a main/named-thread
  affinity guard. The hard cut must make Runtime11 the only dependency/priority/affinity scheduler
  and reduce Editor14 to typed admission, cancellation, result and bounded presentation receipts.

## Static verification

- the workspace declares Rust edition 2021 in root `Cargo.toml`;
- `rustfmt --edition 2021 --check` passed for 47/47 files;
- an edition-2024-only diagnostic still proposes formatting changes in three foreign files, but it
  is not the workspace edition gate and no source was rewritten;
- `git diff --check -- zircon_editor/src/core/jobs` passed;
- the fingerprint was recomputed after the review and still matches the canonical report;
- no dynamic test was run: the managed current editor build remains blocked before Cargo by
  `tools/build-editor.ps1:130`. This also avoids the current quota test helper's `temp_dir()` path,
  which can place artifacts on C: contrary to this session's artifact policy;
- no Rust source changed. Seventeen tracked files and the new folder-backed owners remain foreign
  concurrent work.

The module remains in `pending.md`. Completion still requires current-source Cargo, deterministic
queue/retention/observer/shutdown counters, Runtime11 integration, F0/F4 WPR/xperf/RSS/energy and
independent review. RenderDoc is not an acceptance tool for this CPU scheduling slice.
