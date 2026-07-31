---
related_code:
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/tests/tasks.rs
implementation_files:
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/prelude.rs
plan_sources:
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/01/2026-07-17-task-system-static-review.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-17-task-diagnostics-accuracy.md
tests:
  - python -m unittest tools.tests.test_runtime_job_system_audit
  - runtime_11_job_system_mirror_docs_match_structure_audit_counts
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs::tests::detached_spawn_counts_panicked_tasks_as_completed
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_track_ready_queue_active_and_queue_wait
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_queue_pressure_matrix_drains_without_gauge_leaks
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_reports_conserved_lifecycle_snapshots_during_transitions
  - zircon_runtime/src/tests/tasks.rs::combined_handle_waits_for_all_children_before_propagating_panic
  - zircon_runtime/src/tests/tasks.rs::worker_side_wait_is_reported_as_explicit_wait
  - zircon_runtime/src/tests/tasks.rs::task_diagnostics_distinguish_panics_from_dependency_cancellation
  - cargo test -p zircon_runtime --lib tasks --locked --jobs 1 -- --nocapture --test-threads=1
doc_type: milestone-detail
---

# Runtime11 M3 Task Diagnostics Accuracy

Plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md

Milestone: M3

Status: implementing

Files: [".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/job_system_anchor_inventory.py", "docs/engine-architecture/runtime-architecture-review-m0.md", "docs/engine-architecture/runtime-interface-convergence.md", "docs/plans/zircon_runtime/runtime/11-job-system-task-model.md", "docs/plans/zircon_runtime/runtime/11/2026-07-17-task-diagnostics-accuracy-current-source.md", "docs/plans/zircon_runtime/runtime/11/failure-2026-07-17-task-diagnostics-accuracy.md", "docs/plans/zircon_runtime/runtime/index.md", "docs/zircon_runtime/core/job_system.md", "docs/zircon_runtime/core/runtime/tasks.md", "docs/zircon_runtime/core/tasks.md", "tools/tests/test_runtime_job_system_audit.py", "zircon_runtime/src/core/mod.rs", "zircon_runtime/src/core/runtime/mod.rs", "zircon_runtime/src/core/runtime/tasks/diagnostics.rs", "zircon_runtime/src/core/runtime/tasks/job_handle.rs", "zircon_runtime/src/core/runtime/tasks/job_scheduler.rs", "zircon_runtime/src/core/runtime/tasks/mod.rs", "zircon_runtime/src/core/runtime/tasks/report.rs", "zircon_runtime/src/prelude.rs", "zircon_runtime/src/tests/runtime_absorption/job_system/inventory.rs", "zircon_runtime/src/tests/runtime_absorption/job_system/mirror_docs.rs", "zircon_runtime/src/tests/tasks.rs"]

Date: 2026-07-17

## Scope and inherited change

Performance01 created the canonical failure handoff and left an uncommitted unwind-safe detached-task completion guard plus its regression. Runtime11 preserves that change and completes the lowest scheduler-owned diagnostic contract; it does not absorb Performance01 plan files or the separate Runtime11 thread-budget failure.

## Architecture decision

- `tasks.main_thread_wait_ms` is removed, not aliased. `JobHandle::wait()` is legal on any thread and carries no authoritative caller identity, so the only truthful scheduler-owned name is `tasks.explicit_wait_ms`.
- Work admitted behind prerequisites is derived as `tasks.dependency_waiting`; ready work increments `tasks.queued` immediately before pool submission. Worker start atomically moves one item from queued to `tasks.active` and records cumulative `tasks.queue_wait_ms` plus `tasks.queue_wait_samples`. The four gauges conserve `scheduled = completed + dependency_waiting + queued + active` across dependency release and cancellation.
- First terminal transition decrements active and increments completed. A task panic increments `tasks.panicked`; a dependent closure prevented from launching by an upstream panic increments `tasks.cancelled` without entering queued or active state.
- Combined handles retain their barrier until every child is terminal. The first panic payload is preserved and propagated only after the final child completes, so `wait_all` cannot return while sibling work remains active or under-report explicit synchronization time.
- Lifecycle and duration writers remain atomic-only on the task hot path. Each update is bracketed by an in-flight count and monotonic epoch; overlapping writers retire through an acquire/release handoff before the final zero-writer state becomes visible. `report()` makes at most 16 lock-free read attempts and accepts a snapshot only when no writer overlaps and the epoch stays unchanged. If continuous writers prevent a fresh sample, the reader returns the last confirmed stable snapshot from a report-side cache instead of spinning without a bound or publishing torn values. Queue-wait duration and sample count therefore belong to the same accepted snapshot. No per-task diagnostic mutex, compatibility counter, global caller-thread singleton, or upper-layer inferred queue depth is introduced.

- Dependency continuations are individually panic-contained so a failed callback cannot drop later scheduler/combine callbacks and strand their barriers. All continuations are attempted, terminal observers are then delivered with their existing containment, and the first continuation panic is rethrown after publication.

## Test-first evidence

The initial three public behavior tests for queue/active, worker-side explicit wait, and panic-versus-cancellation were added before the production fields and transitions. At that point the new constants and report fields did not exist, so the current source was intentionally compile-red by construction. A 1/2/4-worker saturation matrix was then added to lock the acceptance pressure shape. Independent successor review later found three missing invariants; focused regressions now cover overlapping-writer publication, dependency-waiting release/cancellation, and panic-first continuation delivery to a surviving combined barrier. The shared managed Cargo lane remains coordinator-owned; no uncoordinated Cargo process is used and no new runtime test pass is claimed until the successor source receives managed evidence.

The non-Cargo audit `python -m unittest tools.tests.test_runtime_job_system_audit` passes 1/1 after the owner inventory hard cut. Focused current-source Cargo evidence and independent review remain required before this record can become accepted.

## Boundary

This slice makes scheduler diagnostics accurate enough for MVP attribution. It does not claim percentile histograms, diagnostics-off product benchmarking, the Editor full-harness natural summary, EventBus backpressure, or Runtime03 frame cadence completion.
