---
related_code:
  - zircon_editor/src/core/sync/mod.rs
  - zircon_editor/src/core/sync/pump.rs
  - zircon_editor/src/core/sync/watch_map.rs
  - zircon_runtime_interface/src/world_sync
tests:
  - zircon_editor/src/core/sync/pump/tests.rs
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

Registration rejects zero tokens and empty invalidation masks without changing either index. Each
binding retains its explicit `depends_on: Vec<WatchKey>` declaration beside the opaque runtime
token, target `ViewInstanceId`, and invalidation mask. The map does not retain a runtime world,
gateway, panel object, or callback.

`WorldSyncPump::watch_view` reuses an existing token only when the view, single explicit key, and
mask are all identical. This makes a repeated lifecycle registration idempotent while preserving
separate dependencies on the same view. Replacing a gateway clears retired bindings before a new
session registers its tokens.

## Frame projection

`WorldWatchMap::project` traverses only `InvalidationBatch::dirty`. Canonical runtime batches
are strictly ascending and unique, so the normal path performs direct token lookups and unions
masks without allocating diagnostic sets. Malformed repeated tokens use the diagnostic slow path,
which reports duplicates and unknown tokens. Neither path scans all registered watches or
recomputes a view projection under a runtime lock.

`WorldSyncPump` consumes the gateway drain once per retained-host frame, publishes immutable
world facts, and submits each projected `ViewDirtySet` through one shared-bus lock acquisition.
Existing dirty views retain their borrowed id through the bus and only a first insertion copies an
id. Gateway generation replacement drops retired tokens before the next drain.

The remaining view-lifecycle registration and retained hierarchy fragment consumer are owned by
[Layout09's recorded failure](../../plans/zircon_editor/editor_layout/09/failure-2026-08-05-retained-hierarchy-dirty-refresh-full-snapshot-fallback.md).
Until that owner registers and unregisters concrete hierarchy view instances, this module does not
claim an end-to-end hierarchy refresh path.

The standalone `editor_world_sync_watch_map` integration gate exercises only the public production
module surface. It avoids enabling unrelated `zircon_editor` library test modules while retaining
the internal invariant tests for replacement, reverse-index cleanup, and invalid registrations.
