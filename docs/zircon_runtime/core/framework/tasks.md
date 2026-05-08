---
related_code:
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
  - zircon_runtime/src/core/framework/tasks/task_pool_kind.rs
  - zircon_runtime/src/core/framework/tasks/task_pool_descriptor.rs
  - zircon_runtime/src/core/framework/tasks/task_poll_budget.rs
  - zircon_runtime/src/core/framework/tasks/task_cancellation_policy.rs
  - zircon_runtime/src/core/framework/tasks/async_task_handle.rs
  - zircon_runtime/src/core/framework/tasks/async_task_descriptor.rs
  - zircon_runtime/src/core/framework/tasks/async_task_state.rs
  - zircon_runtime/src/core/framework/tasks/async_task_status.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_tasks/src/usages.rs
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - cargo test -p zircon_runtime --lib task_framework --locked
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Framework Task Contracts

## Purpose

`zircon_runtime::core::framework::tasks` is the neutral contract layer for Bevy-inspired task-pool vocabulary. It gives runtime modules, asset importers, render preparation, diagnostics, and future app profile wiring a shared way to describe compute, async-compute, and IO task ownership without making the framework layer own a concrete executor.

This slice is intentionally contract-only. The existing `zircon_runtime::core::JobScheduler` remains the concrete rayon-backed runtime primitive. `TasksModule` remains the built-in lifecycle descriptor. Future slices may bridge these contracts into `CoreRuntime`, profile selection, diagnostics, or async executors after their owning sessions coordinate the integration.

## Reference Evidence

Bevy is the primary reference for the split between task-pool categories and concrete executors:

- `dev/bevy/crates/bevy_tasks/src/usages.rs` defines separate compute, async-compute, and IO global pool wrappers.
- `dev/bevy/crates/bevy_tasks/src/task_pool.rs` separates task pool construction, thread naming, scoped work, and main-thread polling details from subsystem code that consumes the pools.

Zircon keeps the same product semantics but does not copy Bevy's global singleton model. The framework contract names the pool kinds and async task diagnostics while leaving runtime ownership with `CoreRuntime` and concrete manager modules.

## Ownership Boundary

The task contracts live under `zircon_runtime::core::framework` because they are shared DTOs and narrow helper types. They do not spawn threads, schedule work, poll futures, or install global executors. Concrete behavior belongs in the runtime kernel, a manager facade, or a subsystem-specific executor owner.

The module deliberately avoids `zircon_app` profile wiring, `zircon_runtime::prelude` exports, asset/resource dependency state, UI focus behavior, and scene ECS scheduling. Those areas have active session ownership and should consume these contracts later through a coordinated slice.

## Data Model

The module is folder-backed so `tasks/mod.rs` stays structural:

- `TaskPoolKind` classifies `Compute`, `AsyncCompute`, and `Io` pools and supplies stable default thread-name stems.
- `TaskPoolDescriptor` describes the desired pool kind, optional worker thread count, and thread name. Worker thread counts clamp to at least one when explicitly provided.
- `TaskPollBudget` records the main-thread poll budget used by future executor pumping. Its default mirrors Bevy's `100` local task-pool ticks per frame while still supporting unlimited polling.
- `TaskCancellationPolicy` records whether dropping a handle should cancel, detach, or finish work during shutdown.
- `AsyncTaskHandle` is a stable numeric identifier for diagnostics and future handle tables.
- `AsyncTaskDescriptor` ties a handle to a pool, label, and cancellation policy.
- `AsyncTaskState` and `AsyncTaskStatus` expose task lifecycle diagnostics, terminal-state detection, poll counts, and failure text.

All types are serializable where appropriate so diagnostics, remote-control, and editor panels can inspect task state without depending on executor internals.

## Behavior

Current behavior is limited to pure helpers and invariants:

- pool descriptors select their default thread names from `TaskPoolKind`,
- explicit worker thread counts clamp to `1` or greater,
- async task status transitions clear stale failure text when moving back to running or completed states,
- poll counts use saturating addition,
- terminal-state helpers classify completed, failed, and cancelled tasks,
- poll budget helpers report remaining per-frame main-thread polls or unlimited polling.

No task is executed by these helpers. A future executor manager should use these contracts as its public description and diagnostic payload instead of exposing concrete rayon, async-executor, or platform thread-pool types.

## Test Coverage

`zircon_runtime/src/core/framework/tests.rs` covers:

- task pool descriptor construction and worker-thread clamping,
- async task descriptors and cancellation policy attachment,
- status transitions from pending through running and failed,
- poll-count recording and default/unlimited poll budgets,
- root module structure so implementation stays in child files rather than `tasks/mod.rs`.

Milestone validation evidence should be recorded in the active Bevy task-pool foundation session note. Full workspace validation remains a later milestone testing-stage concern while other active sessions are changing app, editor, UI, scene, plugin, and asset surfaces.
