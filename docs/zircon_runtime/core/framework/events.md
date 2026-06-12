---
related_code:
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/foundation.rs
  - zircon_runtime/src/core/runtime/events.rs
implementation_files:
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/foundation.rs
plan_sources:
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/core/runtime/tests/events
doc_type: module-detail
---

# Framework Event DTOs

## Purpose

`zircon_runtime::core::framework::events` owns event payload contracts that can cross framework traits, runtime handles, foundation managers, and external consumers without depending on a concrete delivery implementation.

Runtime plan 02 M2.2 split the former core-root event bus fragment into two owners: `EngineEvent` lives here as a neutral DTO, while `EventBus` delivery behavior lives under `zircon_runtime::core::runtime::events`.

## Ownership Boundary

This module may define serializable event DTOs and future framework-level event protocol data. It must not own subscriber storage, delivery locks, pruning, channel fan-out, runtime lifecycle mutation, or service registration.

`core::framework::foundation::EventManager` consumes `EngineEvent` through this namespace. Runtime callers may still use the curated root facade `core::EngineEvent`, but internal runtime code should prefer the owner path.

## API

`EngineEvent` carries:

- `topic: String`
- `payload: serde_json::Value`

The DTO derives `Clone`, `Debug`, `PartialEq`, `Serialize`, and `Deserialize` so it stays suitable for runtime queues, tests, tooling snapshots, and future dynamic/plugin boundaries.

## Validation

`runtime_absorption::root_entries::core_root_splits_event_dto_from_runtime_event_bus` guards that `EngineEvent` is routed through `core::framework::events`, that the old `core/event_bus` fragment is absent, and that the runtime implementation owner exists.
