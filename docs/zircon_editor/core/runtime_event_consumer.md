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

## Performance review status

The current `pump` keeps the active-consumer mutex while calling the gateway drain, validating and
decoding every delivery, and invoking typed/plugin callbacks. It also drains the full backlog on one
retained tick without count/time quotas. This can block reconcile/count calls, deadlock a reentrant
consumer, and transfer arbitrary backlog latency to the editor main thread.

Editor02 owns the linked lock-free snapshot, bounded fair pump, and generation-safe sequence-commit
repair, with Plugins01/runtime transport as a shared implementation dependency. Slow/reentrant and
1k/10k storm tests are pending; no dynamic acceptance is claimed.
