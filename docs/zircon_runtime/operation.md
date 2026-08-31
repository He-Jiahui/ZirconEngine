---
related_code:
  - zircon_runtime/src/operation/mod.rs
  - zircon_runtime/src/operation/context.rs
  - zircon_runtime/src/operation/error.rs
  - zircon_runtime/src/operation/handler.rs
  - zircon_runtime/src/operation/service.rs
  - zircon_runtime/src/operation/task.rs
  - zircon_runtime/src/operation/tests.rs
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/navigation/operation/
implementation_files:
  - zircon_runtime/src/operation/mod.rs
  - zircon_runtime/src/operation/context.rs
  - zircon_runtime/src/operation/error.rs
  - zircon_runtime/src/operation/handler.rs
  - zircon_runtime/src/operation/service.rs
  - zircon_runtime/src/operation/task.rs
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/navigation/operation/handler.rs
  - zircon_runtime/src/navigation/operation/registration.rs
plan_sources:
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/operation/tests.rs
  - zircon_runtime/src/dynamic_api/tests/operation.rs
  - zircon_runtime_interface/src/tests/runtime_operation.rs
  - zircon_plugins/navigation/runtime/src/tests/operation.rs
  - zircon_plugins/navigation/editor/src/tests/operation_command.rs
doc_type: module-detail
---

# Runtime Operation Service

`zircon_runtime::operation` owns runtime-authoritative, handle-based operations that may outlive one editor dispatch. It is independent of editor state and UI. The editor submits versioned payloads through `EditorRuntimeGateway`; only runtime handlers receive `CoreHandle` and mutable `World` access.

## Ownership

- `service.rs` owns handler registration, handle allocation, task storage, polling, and one-time result harvest.
- `task.rs` owns the queued/running/completed/failed task state and progress projection.
- `handler.rs` and `context.rs` define the runtime-only execution contract.
- `runtime_api/operation.rs` owns transport DTOs and function-pointer types; it does not execute operations.
- `dynamic_api/session/operation.rs` is the JSON and owned-buffer ABI adapter. It remains crate-internal because `dynamic_api::exports` is its sibling consumer.
- Feature domains register handlers below their runtime owner. Navigation uses `navigation/operation`; editor crates never receive a `World` reference.

## Lifecycle

`submit` validates ABI and operation identity, allocates a nonzero handle, and records a queued task. The first `poll` exposes `Running` before execution. A later poll takes the payload exactly once, releases the task lock, invokes the handler, and stores either `Completed` with output or `Failed` with diagnostics. Repeated polls observe the stored terminal state without re-executing the handler.

`harvest` rejects nonterminal tasks. A terminal result is removed and returned exactly once; subsequent harvest or poll calls receive an unknown-handle error. Handler execution never occurs while the service mutex is held.

## Editor Transaction Contract

An editor operation factory creates an `EditCommand` that submits through the gateway. Before/after runtime snapshots are stored in the command so undo and redo use a typed restore operation instead of re-running a bake. Any failure after successful submit is conservatively reported as `CommandEffect::Applied`, because the transport cannot prove that a runtime handler left authoritative state unchanged.

The V2 runtime table requires submit, poll, and harvest together. There is no V1 table export, loader fallback, compatibility wrapper, or editor-side direct runtime-world path.
