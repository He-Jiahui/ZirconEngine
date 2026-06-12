---
related_code:
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
implementation_files:
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/pool.rs
  - zircon_runtime/src/core/runtime/tasks/pools.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
  - zircon_runtime/src/core/runtime/tasks/thread_assignment.rs
  - zircon_runtime/src/core/runtime/mod.rs
plan_sources:
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Runtime Task Helpers

## Purpose

`zircon_runtime::core::runtime::tasks` is the runtime-kernel owner for low-level task execution helpers, scheduling facades, and concrete runtime task pools. This subtree owns `spawn_named_thread(...)`, moved out of the former core-root `channel_util.rs` fragment because thread creation is runtime execution behavior; `JobScheduler`, moved out of the former core-root `job_scheduler.rs` fragment; and the concrete `TaskPool` / `TaskPools` rayon-backed executors formerly stored under `core/tasks`.

Runtime plan 02 M2.2 retired the old `zircon_runtime::core::tasks` namespace. Callers use `core::runtime::tasks` for the namespace owner, or the curated `core` / prelude facades for the public task-pool types and `JobScheduler`.

## Ownership Boundary

The runtime task helper layer may own direct OS thread creation, error mapping into `ZirconError`, runtime scheduling facades over the compute pool, and the concrete rayon-backed pool implementation. It does not define neutral DTOs; those remain in `core::framework::tasks` and `core::framework::channel`.

Callers that need the helper should import `core::runtime::tasks::spawn_named_thread`. The `core` root no longer re-exports `spawn_named_thread`.

## API

`spawn_named_thread(...)` wraps `std::thread::Builder::name(...).spawn(...)`, preserving the previous behavior and converting spawn failures to `ZirconError::ThreadSpawn` with the requested thread name included in the error text.

`JobScheduler` is re-exported from `core::runtime` and the curated `core` root facade so scene ECS and prelude callers can continue to use the stable scheduler type while the physical implementation sits under the runtime task owner.

Current production consumers of `spawn_named_thread(...)` are asset event filtering and asset decode worker-pool startup. Both now reach the helper through the runtime owner path.

## Validation

The 2026-06-12 M2.1 migration evidence includes:

- source scans found no remaining `core::channel_util`, `core::types`, root `spawn_named_thread`, root `ChannelSender`, root `ChannelReceiver`, or root `ServiceObject` imports under `zircon_runtime/src`.
- `rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs` passed with 4 tests.
- `cargo check -p zircon_runtime --lib --locked` passed with pre-existing warnings.
- `cargo test -p zircon_runtime --lib runtime_absorption --locked` is not accepted as pass evidence because an unrelated graphics test compile error currently stops the lib-test build.
