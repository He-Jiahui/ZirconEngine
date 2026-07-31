---
related_code:
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events/diagnostics.rs
  - zircon_runtime/src/core/runtime/events/prune.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/events/subscriber.rs
  - zircon_runtime/src/core/runtime/events/topic.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/events.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/framework/events.rs
implementation_files:
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/events/diagnostics.rs
  - zircon_runtime/src/core/runtime/events/prune.rs
  - zircon_runtime/src/core/runtime/events/publish.rs
  - zircon_runtime/src/core/runtime/events/subscribe.rs
  - zircon_runtime/src/core/runtime/events/subscriber.rs
  - zircon_runtime/src/core/runtime/events/topic.rs
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
  - zircon_runtime/src/core/runtime/tests/events/benchmark_evidence.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs
doc_type: module-detail
---

# Runtime Events

## Purpose

`zircon_runtime::core::runtime::events` owns topic-based runtime event delivery. It stores subscribers by topic, serializes same-topic delivery, shares one immutable event allocation across fan-out, applies each subscriber's explicit retention policy, and prunes disconnected subscriptions.

Runtime plan 02 M2.2 moved the former `zircon_runtime::core::event_bus` file and child directory into this runtime owner. `EngineEvent` was split out to `core::framework::events` so the DTO remains a neutral contract while `EventBus` stays runtime behavior.

## Ownership Boundary

This module owns concrete event bus behavior:

- topic subscriber registry storage
- per-topic publish ordering and subscriber snapshots
- shared `Arc<EngineEvent>` fan-out
- lossless, bounded drop-oldest, and latest-only queues
- disconnected-subscriber tracking and topic pruning
- shutdown wake-up and disconnection for subscriptions after the final EventBus owner drops
- exact queue depth/peak, blocked-receiver/publisher, drop, queue-age, publish-duration, and bus-wide aggregate diagnostics for waits on per-topic delivery locks
- poison-safe locking for the topic map, per-topic subscriber snapshot/delivery lock, and per-subscriber queue state

It does not define the event DTO and should import `EngineEvent` from `core::framework::events`.

## Runtime Integration

`CoreRuntimeInner` stores one `EventBus` instance. `CoreRuntime` and `CoreHandle` expose `publish_event(...)`, `subscribe_events(..., policy)`, and `event_bus_diagnostics()`, translating user-facing topics and JSON payloads into framework contracts and making the live production bus metrics observable. Callers must choose a policy; there is no default unbounded subscription path.

The curated `core` root exposes the settled public facade, while the implementation namespace remains `core::runtime::events`.

`EventBusState` owns the poison-safe topic-map lock. Each `EventTopic` owns its subscriber snapshot and delivery lock. Subscribe creates an RAII reservation for the selected topic while holding the map lock, releases the map, and only then waits for the topic delivery lock; last-subscriber removal cannot delete a reserved topic, while unrelated topic lookup/publish is not coupled to that wait. Publish and unsubscribe hold only the selected topic's delivery lock while mutating its subscriber snapshot, so events on different topics can progress independently while same-topic delivery remains ordered.

Each concrete subscription holds a weak state reference and a poison-safe `VecDeque` queue state paired with a condition variable. Admission, drop-oldest replacement, physical dequeue, and queue-depth/peak accounting all commit under the same queue-state lock, so a capacity-one queue cannot report an impossible peak of two. Blocking receivers publish their waiting count before the condition-variable wait. Same-topic publishers first use a poison-safe `try_lock` fast path; only observed contention enters the waiting count and records a delivery-lock wait once acquired, so uncontended publishes do not inflate lock-wait evidence. The final `EventBusState` owner marks every queue inactive, drains it, and wakes all waiters, so blocked, polling, and timed receivers observe the framework `Disconnected` result instead of keeping the runtime alive indefinitely.

`EventBus::new(EventBusDiagnosticsMode::Disabled)` provides the explicit low-overhead path used by performance-sensitive hosts. The disabled sink skips atomics and does not capture queue, publish, or delivery-lock timestamps; its snapshot keeps only live topic/subscriber topology and reports `enabled: false` with metric fields at zero. `EventBus::default()` and `CoreRuntime` keep diagnostics enabled.

## Validation

`core::runtime::tests::events` covers lossless ordering, bounded drop-oldest behavior, latest-only retention, capacity-one peak accuracy, 1/2/5/100-subscriber shared payload identity, queue age/depth, production and disabled diagnostics, zero false lock-wait samples for uncontended publish, same-topic contended lock wait, deterministic blocked-receiver shutdown, exact two-subscriber interleaving under concurrent same-topic publishers, independent different-topic progress, deterministic subscribe/remove reservation overlap, pruning, and source-shape ownership. The structure guards load the folder-backed owners directly, require topic reservation to end the global-map critical section before subscriber delivery locking, reject moving behavior into the root file, reject the removed global delivery lock and deep-clone fan-out, and reject direct production `.lock().unwrap(` use in this owner family.

`core::runtime::tests::events::benchmark_evidence` owns the ignored, managed-only Runtime07 performance matrix. It measures publish p50/p95/max for 1/2/5/100 subscribers across 64 B, 4 KiB, and 256 KiB payloads; compares enabled and disabled diagnostics p95; and records Windows process RSS plus queue age while an 8,704-event producer pressures a capacity-64 paused consumer. The tests hard-gate exact publish/delivery/drop/depth counters, payload size, shared `Arc` identity, disabled-metric zeroing, bounded retention, and final last-64 ordering. RSS and latency values are evidence rather than machine-dependent thresholds.

Managed acceptance first runs the ordinary behavior/structure gate against the same source manifest; ignored benchmark tests remain skipped in this gate:

```powershell
cargo +1.94.1 test -p zircon_runtime --lib core::runtime::tests::events:: --locked --jobs 1 -- --nocapture --test-threads=1
```

It then runs the performance evidence command twice and records every `EVENTBUS_BENCH_V1` line before closing the Runtime07 failure handoff:

```powershell
cargo +1.94.1 test -p zircon_runtime --lib event_bus_runtime07_ --locked --jobs 1 -- --ignored --nocapture --test-threads=1
```

The ordinary library suite leaves these tests ignored, so broad correctness validation does not silently absorb a performance workload. Allocation acceptance is expressed as one shared `Arc` identity across every fan-out subscriber plus the bounded retained-payload byte count and process RSS under paused-consumer pressure. This directly guards the removed per-subscriber deep clone and bounded-retention properties without claiming allocator-internal byte accounting that the runtime does not own.

`runtime_absorption::root_entries::core_root_splits_event_dto_from_runtime_event_bus` guards the M2.2 hard cutover: no `core/event_bus.rs`, no `core/event_bus/`, `framework::events` is declared, and `runtime::events` owns `EventBus`.

Runtime 15 M3 F2 lock poison recovery guard / `runtime_15_f2_lock_poison_recovery_guard_core_min_cargo_passed_full_sweep_pending` adds `structure_convention/lock_poison_policy.rs::runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus` as a cross-owner regression guard for EventBus and scene level lock helpers. `code_review_findings/p0_robustness.rs::review_f2_scene_eventbus_locks_recover_after_poison` mirrors the review table status as `scene/EventBus poison-safe lock recovery complete` and locks the module-local `level_system_accessors_recover_poisoned_state_locks` behavior test.

2026-06-22 F2 validation for the poison-safe lock slice: scoped rustfmt/check passed for the touched EventBus files and guards; production static scan found no direct `.lock().unwrap(` in `core/runtime/events.rs` or its publish/subscribe/prune owners. Focused Cargo validation was attempted through the Runtime 07 scene poison test and timed out during compilation, so no package-level Cargo pass is claimed for this slice.

2026-06-27 F2 status closure validation: the new review guard first failed on missing `runtime_15_f2_lock_poison_recovery_guard_core_min_cargo_passed_full_sweep_pending` docs/status anchors. After the status mirrors were synced, `level_system_accessors_recover_poisoned_state_locks`, `review_f2_scene_eventbus_locks_recover_after_poison`, `runtime_15_f2_lock_poison_recovery_guard_covers_scene_and_eventbus`, `runtime_15_code_review_findings_tests_are_folder_backed`, and `status_output_tables` passed under core-min focused Cargo with target dir `E:\cargo-targets\zircon-runtime-f2-review-status-0627`. Full Runtime 15 Cargo sweep remains pending.
