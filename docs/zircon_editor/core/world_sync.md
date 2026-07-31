---
related_code:
  - zircon_editor/src/core/sync/mod.rs
  - zircon_editor/src/core/sync/watch_map.rs
  - zircon_runtime_interface/src/world_sync
tests:
  - zircon_editor/src/core/sync/watch_map/tests.rs
  - zircon_editor/tests/editor_world_sync_watch_map.rs
  - tools/tests/test_editor02_world_sync_watch_map_contract.py
plan_sources:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
doc_type: module-detail
---

# Editor world sync projection

`core::sync` owns editor-side state derived from the transport-neutral world-sync protocol. Runtime subscription ownership remains in `zircon_runtime::scene::inspection`; UI views never enter the runtime subscription table.

## Watch map authority

`WorldWatchMap` is session-scoped. Its `by_token` index is the registration authority and `by_view` is the reverse lifecycle index used when a view closes. Rebinding a token removes the old reverse relation before publishing the new binding. Unbinding a view or draining a session returns sorted runtime tokens so the gateway layer can issue deterministic unwatch requests.

Registration rejects zero tokens and empty invalidation masks without changing either index. The map stores only runtime `WatchToken`, editor `ViewInstanceId`, and the view invalidation mask; it does not retain a runtime world, gateway, panel object, or callback.

## Frame projection

`WorldWatchMap::project` traverses only `InvalidationBatch::dirty`. It deduplicates malformed repeated tokens, performs direct token lookups, unions masks per view through `ViewDirtySet`, and reports unknown tokens for lifecycle diagnostics. It does not scan all registered watches or recompute a view projection under a runtime lock.

The resulting `ViewDirtySet` is consumed by the editor frame pump. Gateway watch/unwatch/drain calls and retained-host wiring remain separate M2 work; this module does not claim that the full M2 path is active.

The standalone `editor_world_sync_watch_map` integration gate exercises only the public production
module surface. It avoids enabling unrelated `zircon_editor` library test modules while retaining
the internal invariant tests for replacement, reverse-index cleanup, and invalid registrations.
