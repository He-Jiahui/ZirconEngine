---
related_code:
  - zircon_editor/src/core/runtime_event_consumer/mod.rs
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
implementation_files:
  - zircon_editor/src/core/runtime_event_consumer/error.rs
  - zircon_editor/src/core/runtime_event_consumer/registration.rs
  - zircon_editor/src/core/runtime_event_consumer/host.rs
  - zircon_editor/src/core/runtime_event_consumer/pump.rs
  - zircon_editor/src/core/gateway/contract.rs
  - zircon_editor/src/core/gateway/handle.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-runtime-event-consumer-unbounded-pump-lock.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/tests/runtime_event_consumer.rs
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump.rs
  - zircon_plugins/navigation/editor/src/tests.rs
doc_type: module-detail
---

# Editor Runtime Event Consumer

The editor consumer host is a PIE-scoped protocol owner. Plugin registrations provide a data-only `PluginEventConsumerManifest` plus a typed `EditorRuntimeEventConsumerState`. The generic registration decodes JSON into the declared payload type and owns the plugin state behind a recoverable mutex.

Entering PIE allocates a monotonic play-session generation that is independent from the long-lived
runtime handle. The typed state receives that generation, while each drained delivery is validated
against the actual runtime session handle, subscription, event id, payload schema, and a sequence
newer than the last accepted value. A rejected delivery never updates plugin state.

The retained host refreshes the capability snapshot on every active PIE tick. The consumer host
first unsubscribes capabilities that became disabled, then subscribes newly enabled registrations.
Only after reconciliation does the editor gateway advance the runtime frame and drain deliveries.
This ordering prevents a disabled debug consumer from keeping the runtime mirror capture path active
for an extra frame. Failed additions roll back only additions from that reconciliation. An
unsubscribe response of `false` or a gateway error is surfaced as cleanup failure while the active
subscription and typed state remain owned locally, so the host can retry instead of orphaning a
runtime reader.

Exiting PIE unsubscribes every active handle and calls the typed state cleanup hook only for handles
the runtime confirms removed. If any removal fails, the play generation and failed subscriptions
remain active until a later retry succeeds. The retained host tick is the single ordering owner
for capability reconciliation, runtime advancement, and delivery pumping; plugins do not create
polling threads or own an editor-side `World`.

Consumer implementations expose a typed error associated with their state. The registration owner
preserves JSON decode errors and plugin-state errors as sources instead of flattening either public
boundary to `String`.

`EditorPluginRegistrationReport` carries the typed registry and validates that its data manifests exactly match `EditorPluginDescriptor`. Host extension registration installs that registry alongside the plugin's editor extensions.

## Bounded pump and pressure reporting

`pump` now snapshots stable consumer id, registration, subscription, and generation while holding
the registry mutex, then releases that mutex before gateway drain, JSON decoding, and typed/plugin
callbacks. A reentrant callback may therefore inspect host state without waiting on the pump's
registry lock. Concurrent or recursively reentered pumps return an empty report instead of creating
a second delivery owner. A callback that tries to mutate the consumer lifecycle receives the typed
`LifecycleMutationBusy` error instead of recursively taking its own typed-state mutex. Pump and
lifecycle mutation acquire one atomic execution state, so lifecycle cannot pass a check and then
race unsubscribe/end-session against a newly started pump.
Subscribe, unsubscribe, begin-session, and end-session calls also execute outside the active-map
critical section, so a slow gateway or lifecycle callback cannot pin host observation behind that
map lock.

Every drained delivery moves into the matching generation's editor-owned pending queue before
callbacks run. `EditorRuntimeEventPumpBudget` applies global event, per-consumer event, elapsed-time,
and slow-callback thresholds. A round-robin cursor changes the first consumer on the next tick, so a
hot subscription cannot monopolize a small budget. Gateway, validation, and payload failures retain
the first typed error but continue visiting later consumers before returning it, preventing one bad
consumer from pinning the fair-pump head. Successful sequence writeback is conditional on the same
consumer generation and subscription still being active.

`EditorRuntimeEventPumpReport` exposes applied, drained, deferred, dropped, slow-callback, queue-depth,
and pending-sequence-span pressure. The span is a backlog sequence range, not wall-clock age. Invalid
or consumer-rejected deliveries return their typed error and increment dropped explicitly; every
remaining delivery stays queued in order for a later tick. Delivery payload ownership moves directly
into the typed callback without cloning the JSON value.

The current runtime/session transport still returns one fully encoded and decoded delivery vector.
Therefore callback elapsed budgets are bounded, but transport encode/decode is not yet a complete
frame-time bound. Plugins01 owns the required count/byte bounded drain contract; see
`docs/plans/zircon_plugins/01/failure-2026-07-22-plugin-event-drain-frame-budget.md`. This Editor02
failure remains open until that dependency has managed dynamic evidence.

The focused static contract and Rust regressions cover count deferral, order, normal/error-path
round-robin fairness, reentrant observation, concurrent typed-busy lifecycle mutation, and slow
callbacks. Independent re-review is clean at Critical/Important/Minor=`0/0/0`. Managed Cargo and
Plugins01 bounded transport evidence remain required before the linked failure can be returned as
fixed.
