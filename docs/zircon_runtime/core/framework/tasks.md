---
related_code:
  - zircon_runtime/src/core/framework/tasks/mod.rs
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/framework/tasks/mod.rs
  - zircon_runtime/src/core/framework/tasks/task_pool_kind.rs
  - zircon_runtime/src/core/framework/tasks/task_pool_descriptor.rs
  - zircon_runtime/src/core/framework/tasks/task_poll_budget.rs
  - zircon_runtime/src/core/framework/tasks/task_cancellation_policy.rs
  - zircon_runtime/src/core/framework/tasks/async_task_handle.rs
  - zircon_runtime/src/core/framework/tasks/async_task_descriptor.rs
  - zircon_runtime/src/core/framework/tasks/async_task_state.rs
  - zircon_runtime/src/core/framework/tasks/async_task_status.rs
  - zircon_runtime/src/core/framework/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/tasks/mod.rs
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/framework/tasks/task_pool_kind.rs
  - zircon_runtime/src/core/framework/tasks/task_pool_descriptor.rs
  - zircon_runtime/src/core/framework/tasks/task_poll_budget.rs
  - zircon_runtime/src/core/framework/tasks/task_cancellation_policy.rs
  - zircon_runtime/src/core/framework/tasks/async_task_handle.rs
  - zircon_runtime/src/core/framework/tasks/async_task_descriptor.rs
  - zircon_runtime/src/core/framework/tasks/async_task_state.rs
  - zircon_runtime/src/core/framework/tasks/async_task_status.rs
  - zircon_runtime/src/core/framework/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_tasks/src/usages.rs
  - docs/plans/zircon_runtime/runtime/11/failure-2026-07-13-editor-full-harness-runtime-thread-budget.md
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/tests/tasks.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
  - tools/tests/test_runtime_job_system_audit.py
  - cargo test -p zircon_runtime --lib task_framework --locked
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Framework Task Contracts

## Purpose

`zircon_runtime::core::framework::tasks` is the neutral contract layer for Bevy-inspired task-pool vocabulary. It gives runtime modules, asset importers, render preparation, diagnostics, and future app profile wiring a shared way to describe compute, async-compute, and IO task ownership without making the framework layer own a concrete executor.

This framework layer is intentionally contract-only. The concrete rayon-backed bridge lives in `zircon_runtime::core::runtime::tasks`, where `TaskPools` owns compute, async-compute, and IO execution. Default runtimes share one process-wide task owner; explicitly constructed pools remain isolated. `TasksModule` remains the built-in lifecycle descriptor.

## Reference Evidence

Bevy is the primary reference for the split between task-pool categories and concrete executors:

- `dev/bevy/crates/bevy_tasks/src/usages.rs` defines separate compute, async-compute, and IO global pool wrappers.
- `dev/bevy/crates/bevy_tasks/src/task_pool.rs` separates task pool construction, thread naming, scoped work, and main-thread polling details from subsystem code that consumes the pools.

Zircon keeps the same product semantics without exposing Bevy-style global wrapper types to framework consumers. The framework contract names pool kinds and async task diagnostics; the concrete runtime layer owns the process-default `OnceLock<TaskPools>` and explicit isolated-owner construction.

## Ownership Boundary

The task contracts live under `zircon_runtime::core::framework` because they are shared DTOs and narrow helper types. They do not spawn threads, schedule work, poll futures, or install global executors. Concrete behavior belongs in `zircon_runtime::core::runtime::tasks`, runtime manager facades, or subsystem-specific executor owners.

The framework module deliberately avoids `zircon_app` profile wiring, concrete prelude policy, asset/resource dependency state, UI focus behavior, and scene ECS scheduling. `zircon_runtime::core::runtime::tasks` and the runtime prelude may expose the concrete pool facade, while higher-level systems should consume the framework contracts through coordinated runtime-owned slices.

## Data Model

The module is folder-backed so `tasks/mod.rs` stays structural:

- `TaskPoolKind` classifies `Compute`, `AsyncCompute`, and `Io` pools and supplies stable default thread-name stems.
- `TaskPoolDescriptor` describes the desired pool kind, optional worker thread count, and thread name. Worker thread counts clamp to at least one when explicitly provided.
- `TaskPollBudget` records the main-thread poll budget used by future executor pumping. Its default mirrors Bevy's `100` local task-pool ticks per frame while still supporting unlimited polling.
- `TaskCancellationPolicy` records whether dropping a handle should cancel, detach, or finish work during shutdown.
- `AsyncTaskHandle` is a stable numeric identifier for diagnostics and future handle tables.
- `AsyncTaskDescriptor` ties a handle to a pool, label, and cancellation policy.
- `AsyncTaskState` and `AsyncTaskStatus` expose task lifecycle diagnostics, terminal-state detection, poll counts, and failure text.
- `ParallelSliceExecutor` is a narrow blocking slice-parallelism contract. Framework algorithms can request bounded parallel work without importing the concrete runtime pool or Rayon.

All types are serializable where appropriate so diagnostics, remote-control, and editor panels can inspect task state without depending on executor internals.

## Behavior

Current behavior is limited to pure helpers and invariants:

- pool descriptors select their default thread names from `TaskPoolKind`,
- explicit worker thread counts clamp to `1` or greater,
- async task status transitions clear stale failure text when moving back to running or completed states,
- poll counts use saturating addition,
- terminal-state helpers classify completed, failed, and cancelled tasks,
- poll budget helpers report remaining per-frame main-thread polls or unlimited polling.
- `TaskPool` implements `ParallelSliceExecutor` in the runtime task owner, so framework algorithms execute on the caller-supplied pool. The framework layer never creates or discovers the process-default owner itself.

Framework contracts do not own task execution. Calling `ParallelSliceExecutor::parallel_for(...)` delegates execution to the supplied implementation; `TaskPools` consumes the remaining descriptors and kinds as its public description. Future async task managers should use the async task contracts as diagnostic payloads instead of exposing concrete rayon, async-executor, or platform thread-pool types.

## Test Coverage

`zircon_runtime/src/core/framework/tests.rs` covers:

- task pool descriptor construction and worker-thread clamping,
- async task descriptors and cancellation policy attachment,
- status transitions from pending through running and failed,
- poll-count recording and default/unlimited poll budgets,
- root module structure so implementation stays in child files rather than `tasks/mod.rs`.
- source cubemap explicit-executor construction preserving the synchronous builder output contract,
- the Runtime 11 audit proving that source mip generation has no direct Rayon reference and that the only production Rayon owners remain `pool.rs` and `parallel_for.rs`.

Milestone validation evidence should be recorded in the active Bevy task-pool foundation session note. Full workspace validation remains a later milestone testing-stage concern while other active sessions are changing app, editor, UI, scene, plugin, and asset surfaces.
