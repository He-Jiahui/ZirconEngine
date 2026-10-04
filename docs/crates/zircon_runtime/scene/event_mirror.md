---
related_code:
  - zircon_runtime/src/scene/event_mirror/mod.rs
  - zircon_runtime/src/scene/world/event_mirror.rs
  - zircon_runtime/src/plugin/extension_registry/register/event_registration.rs
implementation_files:
  - zircon_runtime/src/scene/event_mirror/error.rs
  - zircon_runtime/src/scene/event_mirror/registration.rs
  - zircon_runtime/src/scene/event_mirror/subscription.rs
  - zircon_runtime/src/scene/world/event_mirror.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/world/derived_state.rs
plan_sources:
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime/src/scene/tests/ecs_event_mirror.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/world_driver.rs
  - zircon_plugins/navigation/runtime/src/tests/runtime_mirror.rs
doc_type: module-detail
---

# Runtime ECS Event Mirror

The event mirror exposes selected serializable ECS events to a host without creating a second event catalog. `RuntimeEventMirrorRegistration::typed` binds the stable event id and payload schema to the concrete ECS event type. A subscription uses the existing `EventSubscription<T>` cursor, so activation starts at the current queue position and never replays historical frames. Events emitted after the frame's `UpdateEvents` system enter the ECS `next` generation and become host-drainable after the following runtime tick; draining never advances or flushes ECS generations.

Registration is opt-in through `RuntimeExtensionRegistry::register_mirrored_event`. Ordinary plugin events keep using `register_event` and need not implement `Serialize`. Duplicate ids, unknown ids, schema mismatches, disconnected drains, and serialization failures are typed errors.

Reader-count callbacks run after mirror connect and disconnect. The registry keeps a dedicated count
per stable mirror event id; ordinary ECS `EventReader<T>` instances are deliberately excluded.
Navigation uses this generic callback to enable `NavigationDebugCapture` only while at least one
host mirror reader exists. Multiple mirror readers are reference counted, rollback decrements a
failed connection, and debug payload generation stops after the final mirror subscription
disconnects.

Unsubscribe is transactional. A failed disconnect leaves the subscription owned by the caller; a
reader-count callback failure reconnects the ECS subscription and restores the mirror-specific
count before returning the typed error. Dynamic sessions remove their handle map entry only after
the World confirms disconnection, so callers can retry and cannot create orphan debug readers.

The runtime dynamic session owns subscription handles and per-subscription sequence numbers. The `World` owns only the typed mirror catalog, so editor code never owns or mutates the runtime ECS world.

The schedule runner executes each declared internal system once in its ordered stage. Stage teardown
may run only still-dirty derived-state systems; it must not replay unconditional internal systems
such as `UpdateEvents` or `ApplyDeferred`. This keeps the event-generation rotation at exactly once
per runtime tick and makes the next-tick mirror contract stable.
