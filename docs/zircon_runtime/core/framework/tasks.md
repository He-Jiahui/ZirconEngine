---
related_code:
  - zircon_runtime/src/core/framework/tasks/mod.rs
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/mipmap.rs
  - zircon_runtime/src/core/runtime/tasks/parallel_for.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/framework/mod.rs
implementation_files:
  - zircon_runtime/src/core/framework/tasks/mod.rs
  - zircon_runtime/src/core/framework/tasks/parallel_slice_executor.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
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

`zircon_runtime::core::framework::tasks` contains only the backend-neutral
blocking data-parallelism contract used by framework algorithms.

The concrete rayon-backed bridge and every task lifecycle contract live in
`zircon_runtime::core::runtime::tasks`. Runtime11 owns pool selection,
admission, identity, cancellation, status, dependencies, waits, and shutdown.
The framework layer cannot create or discover an execution owner.

## Reference Evidence

Bevy is the primary reference for the split between task-pool categories and concrete executors:

- `dev/bevy/crates/bevy_tasks/src/usages.rs` defines separate compute, async-compute, and IO global pool wrappers.
- `dev/bevy/crates/bevy_tasks/src/task_pool.rs` separates task pool construction, thread naming, scoped work, and main-thread polling details from subsystem code that consumes the pools.

Zircon transfers only the narrow algorithm boundary. Bevy task-pool category
and handle details do not justify a second framework DTO state machine beside
the Runtime scheduler.

## Ownership Boundary

`ParallelSliceExecutor` lives under `zircon_runtime::core::framework` because
environment and render-preparation algorithms need blocking slice work without
depending on `TaskPool` or Rayon. It does not expose a pool kind, task identity,
mutable status, cancellation policy, or scheduler handle.

`TaskId`, `TaskDescriptor`, `TaskState`, `TaskStatus`, `TaskHandle`,
`TaskCancellationPolicy`, `TaskPoolKind`, and `TaskPoolDescriptor` are
Runtime11 contracts. `TaskPollBudget` and the old `AsyncTask*` files were
deleted in the 2026-08-28 hard cut because no executor consumed the budget and
the duplicate status model could disagree with executable completion.

## Data Model

The folder contains one implementation file and one structural root:

- `ParallelSliceExecutor` requests bounded mutable-slice work or stable indexed
  output without importing the concrete runtime pool or Rayon.
- `tasks/mod.rs` declares and re-exports that trait only.

## Behavior

`TaskPool` implements `ParallelSliceExecutor` in the runtime task owner, so
framework algorithms execute on a caller-supplied owner. Indexed output keeps
source order, and the trait's default indexed adapter is bounded and serial.

Calling `parallel_for(...)` or `parallel_map_indices(...)` delegates execution
to the supplied implementation. Async work must enter Runtime11 through
`TaskHandle`, `TaskGraphScope`, or `JobScheduler`, never through a framework
status copy.

## Test Coverage

`zircon_runtime/src/core/framework/tests.rs` covers:

- the root module exporting only the neutral parallel trait,
- source cubemap explicit-executor construction preserving the synchronous builder output contract,
- the Runtime 11 audit proving that source mip generation has no direct Rayon reference and that the only production Rayon owners remain `pool.rs` and `parallel_for.rs`.

Milestone validation evidence should be recorded in the active Bevy task-pool foundation session note. Full workspace validation remains a later milestone testing-stage concern while other active sessions are changing app, editor, UI, scene, plugin, and asset surfaces.
