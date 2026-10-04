---
related_code:
  - zircon_runtime/src/scene/inspection/subscription.rs
  - zircon_runtime_interface/src/world_sync/watch.rs
  - zircon_runtime_interface/src/world_sync/invalidation.rs
plan_sources:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
tests:
  - zircon_runtime/src/scene/inspection/subscription/tests.rs
  - zircon_runtime/tests/runtime_world_sync_subscription_table.rs
  - tools/tests/test_editor02_world_sync_subscription_table_contract.py
doc_type: module-detail
---

# World Sync Subscription Table

`SubscriptionTable` is the runtime-session authority for editor world watches. It owns opaque token
allocation, reverse token ownership, four typed routing indexes, and the bounded one-frame
fact/dirty state. It stores no editor view identifiers and is discarded with the runtime session.

## Watch Lifecycle

- `watch` allocates a non-zero token for every registration. Multiple consumers of the same key
  receive distinct tokens.
- `by_token` is the unwatch authority. Registration and removal update exactly one typed index:
  `world_tokens`, `subtree_tokens`, `component_tokens`, or `asset_tokens`. The removed generic
  `WatchKey -> tokens` map is not retained as a compatibility path.
- `unwatch` removes the reverse record and its typed route, then revokes any pending dirty mark.
  Repeating an unwatch is an explicit `false` result.
- `flush` emits dirty tokens in deterministic token order and coalesced facts in first-key order,
  then clears only the one-frame pending state. An empty frame returns `None`.

## Mutation Throats

`record_fact` handles facts that already carry enough runtime identity. Spawn, despawn, and
reparent facts hit `WorldStructure` plus the entity's current subtree ancestry. One fact builds one
cycle-guarded ancestor chain and probes only subscribed roots on that chain; watch count does not
multiply parent traversal. Scene load/unload facts hit `WorldStructure` and one asset route.
`AssetReloadApplied` has no resource id in the stable DTO, so it visits only `asset_tokens`; it does
not scan world, subtree, or component keys and does not collect a temporary token vector.

Three explicit helpers cover mutation information that the current `WorldFact` DTO intentionally
does not duplicate:

- `invalidate_subtree(world, entity)` marks every subscribed root in the current parent chain.
  Reparent/despawn throats call it before mutation to retain old ancestry, then successful mutation
  records the fact against the new/current world. Failed attempts may produce a same-generation
  conservative dirty mark, which query generation hints short-circuit.
- `invalidate_component_type(type_name)` uses `BTreeMap<String, _>::get(&str)`, so the mutation
  throat does not allocate a lookup `String`. False positives from an acquired mutable reference
  are allowed by the Editor02 plan.
- `invalidate_asset(resource_id)` is called when a precise resource id is available.

Ancestor traversal has one visited set per fact. Malformed imported parent cycles terminate
without hanging the runtime or inventing a match outside the visited chain.

## Bounded fact queue

`SubscriptionTableLimits` bounds unique pending fact count, estimated bytes, and age in world
generations. The byte estimate includes the fact, semantic coalesce key, and index slot. Entity,
scene, and aggregate reload keys update their existing slot; aggregate reload counters saturating-
add rather than losing intermediate apply/fail/stale counts.

When count or byte admission overflows, the fact is not appended and
`SubscriptionTableDiagnostics::overflowed_facts` advances. The mutation route has already marked
the affected world/subtree/asset tokens dirty, so consumers perform a query resync rather than
trusting an incomplete fact stream. An age breach marks world structure dirty once for that frame.
Diagnostics retain ancestor walks/nodes/visited allocations, direct key probes, matched tokens,
coalesced and overflowed facts, pending peaks, oldest age, and a cumulative overflow flag.

## Current Boundary

This slice supplies the M2.1 table and the separate editor watch-map support layer only.
`RuntimeDynamicSession` ownership, world mutation wiring, frame-end flush, gateway methods, and
retained-host pump remain separate M2.1/M2.2 work. Their absence is not hidden by marking the parent
milestone complete.

The focused public gate is the standalone `runtime_world_sync_subscription_table` integration test.
It compiles the production runtime library without enabling unrelated `zircon_runtime` lib-test
modules. Its ignored 100k case proves a single borrowed component lookup, 100k aggregate fact
coalescing, and bounded pending peak; the broader unit-module gate remains required after current
cross-module test drift clears.
