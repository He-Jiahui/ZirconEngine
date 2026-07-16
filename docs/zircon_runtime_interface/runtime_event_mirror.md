---
related_code:
  - zircon_runtime_interface/src/runtime_api/plugin_event_mirror.rs
  - zircon_runtime_interface/src/runtime_api/api_table.rs
implementation_files:
  - zircon_runtime_interface/src/runtime_api/plugin_event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
plan_sources:
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/dynamic_api/tests/api_table.rs
  - zircon_runtime/src/dynamic_api/tests/linked_plugins.rs
doc_type: module-detail
---

# Runtime Plugin Event Mirror ABI

`ZrRuntimeApiV2` is the sole runtime service table. It owns the base session functions plus
subscribe, unsubscribe, and drain plugin events as a required three-function group alongside the
operation lifecycle. Loaders resolve only `zircon_runtime_get_api_v2`; a missing symbol, incomplete
table, or invalid version is rejected without downgrade or compatibility fallback.

Subscribe accepts a JSON `ZrRuntimePluginEventSubscribeRequestV1` containing the ABI version, stable event id, and payload schema. Drain returns `ZrRuntimePluginEventDeliveryBatchV1`. Every delivery carries the play-session id, subscription handle, event id, payload schema, strictly increasing per-subscription sequence, and JSON payload.

Drain observes only the ECS event generation made current by the runtime frame schedule. It does not
flush the World or rotate event generations, so an event emitted after `UpdateEvents` becomes
visible to the host after the next tick. This preserves the same no-history cursor semantics used
by in-process ECS readers.

Runtime-owned output buffers carry a dedicated owner token and free callback. `RuntimeSession` always frees decoded output, rejects a mismatched ABI version, and forwards the typed delivery list to the editor host.
