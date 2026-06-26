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
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
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
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
plan_sources:
  - user: 2026-06-12 runtime architecture implementation from docs/plans/zircon_runtime/runtime
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/core/runtime/tests/events
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
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
- poison-safe locking for the subscriber registry and delivery serialization lock

It does not define the event DTO and should import `EngineEvent` from `core::framework::events`.

## Runtime Integration

`CoreRuntimeInner` stores one `EventBus` instance. `CoreRuntime` and `CoreHandle` expose `publish_event(...)` and `subscribe_events(...)`, translating user-facing topics and JSON payloads into framework `EngineEvent` values.

The curated `core` root still exposes `EventBus` for compatibility with the settled public facade, but the implementation namespace is `core::runtime::events`.

`EventBus` keeps lock ownership in the root owner file through private `lock_subscribers()` and `lock_delivery()` helpers. Publish, subscribe, and prune code call those helpers instead of opening the `Mutex` directly, so a poisoned subscriber registry or delivery lock is recovered with the owned state instead of panicking the runtime event path.

## Validation

`core::runtime::tests::events` covers publish/subscribe behavior and source-shape ownership. The structure guards load `core/runtime/events.rs` and `core/runtime/events/{subscribe,publish,failure,prune}.rs` directly, reject moving publish, subscribe, prune, or failed-subscriber matching back into the root owner file, and reject direct production `.lock().unwrap(` use in this owner family.

`runtime_absorption::root_entries::core_root_splits_event_dto_from_runtime_event_bus` guards the M2.2 hard cutover: no `core/event_bus.rs`, no `core/event_bus/`, `framework::events` is declared, and `runtime::events` owns `EventBus`.

Runtime 15 M3 F2 lock poison recovery guard / `runtime_15_f2_lock_poison_recovery_guard_static_passed_cargo_deferred` adds `structure_convention/lock_poison_policy.rs::runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus` as a cross-owner regression guard for EventBus and scene level lock helpers.

2026-06-22 F2 validation for the poison-safe lock slice: scoped rustfmt/check passed for the touched EventBus files and guards; production static scan found no direct `.lock().unwrap(` in `core/runtime/events.rs` or its publish/subscribe/prune owners. Focused Cargo validation was attempted through the Runtime 07 scene poison test and timed out during compilation, so no package-level Cargo pass is claimed for this slice.
