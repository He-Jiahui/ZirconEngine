---
related_code:
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events/failure.rs
  - zircon_runtime/src/core/runtime/events/prune.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/core/framework/events.rs
implementation_files:
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events/failure.rs
  - zircon_runtime/src/core/runtime/events/prune.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
plan_sources:
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/core/runtime/tests/events
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
doc_type: module-detail
---

# Runtime Events

## Purpose

`zircon_runtime::core::runtime::events` owns topic-based runtime event delivery. It stores subscriber lists, snapshots delivery targets, serializes publish fan-out with a delivery lock, and prunes closed receivers after failed sends.

Runtime plan 02 M2.2 moved the former `zircon_runtime::core::event_bus` file and child directory into this runtime owner. `EngineEvent` was split out to `core::framework::events` so the DTO remains a neutral contract while `EventBus` stays runtime behavior.

## Ownership Boundary

This module owns concrete event bus behavior:

- topic subscriber registry storage
- publish snapshot and fan-out
- failed-subscriber tracking
- topic pruning
- small-slice fast paths for subscribe, publish, and prune

It does not define the event DTO and should import `EngineEvent` from `core::framework::events`.

## Runtime Integration

`CoreRuntimeInner` stores one `EventBus` instance. `CoreRuntime` and `CoreHandle` expose `publish_event(...)` and `subscribe_events(...)`, translating user-facing topics and JSON payloads into framework `EngineEvent` values.

The curated `core` root still exposes `EventBus` for compatibility with the settled public facade, but the implementation namespace is `core::runtime::events`.

## Validation

`core::runtime::tests::events` covers publish/subscribe behavior and source-shape ownership. The structure guards load `core/runtime/events.rs` and `core/runtime/events/{subscribe,publish,failure,prune}.rs` directly, and reject moving publish, subscribe, prune, or failed-subscriber matching back into the root owner file.

`runtime_absorption::root_entries::core_root_splits_event_dto_from_runtime_event_bus` guards the M2.2 hard cutover: no `core/event_bus.rs`, no `core/event_bus/`, `framework::events` is declared, and `runtime::events` owns `EventBus`.
