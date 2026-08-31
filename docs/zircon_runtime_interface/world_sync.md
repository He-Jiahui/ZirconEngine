---
related_code:
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_interface/src/world_sync/mod.rs
  - zircon_runtime_interface/src/world_sync/query.rs
  - zircon_runtime_interface/src/world_sync/watch.rs
  - zircon_runtime_interface/src/world_sync/invalidation.rs
  - zircon_runtime_interface/src/resource/resource_id.rs
  - zircon_runtime_interface/src/tests/world_sync_contracts.rs
implementation_files:
  - zircon_runtime_interface/src/world_sync/mod.rs
  - zircon_runtime_interface/src/world_sync/query.rs
  - zircon_runtime_interface/src/world_sync/watch.rs
  - zircon_runtime_interface/src/world_sync/invalidation.rs
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/engine-code-structure-convention.md
tests:
  - zircon_runtime_interface/src/tests/world_sync_contracts.rs
  - cargo test -p zircon_runtime_interface --locked
doc_type: module-detail
---

# World synchronization contracts

## Purpose

`zircon_runtime_interface::world_sync` defines the transport-neutral data contract used to query runtime world state, subscribe to invalidation families, and drain per-frame invalidation facts. It is shared by the planned in-process and serialized `EditorRuntimeGateway` implementations, so neither transport may invent its own query or watch schema.

The interface crate owns DTOs only. Runtime world storage, subscription matching, generation mutation, editor view identities, message-bus delivery, and transport I/O remain in their runtime/editor owners.

## Module ownership

- `query.rs` owns the typed `WorldQuery` projection enum, component filters/selectors, deterministic entity rows, runtime hierarchy rows, focused Inspector field rows, and `WorldQueryResult`.
- `watch.rs` owns typed `WatchKey` values, registrations, and runtime-issued opaque `WatchToken` identities.
- `invalidation.rs` owns `WorldFact`, the stable dynamic-scene reload summary, and `InvalidationBatch`.
- `mod.rs` is a navigational façade and contains no query, matching, or serialization behavior.

This four-file shape follows Editor02 M1.1 and keeps future subscription-table or gateway implementations out of the interface contract.

## Query model

`WorldQuery` is a tagged enum. `Components(ComponentWorldQuery)` carries fully qualified component filters/selectors; `Hierarchy(WorldHierarchyQuery)` requests the runtime-owned inspection hierarchy; `InspectionFields(WorldInspectionFieldsQuery)` requests every editor-visible reflected field for one focused entity. The enum prevents hierarchy and Inspector requests from carrying meaningless component filters or selectors. `EntityRow` uses `BTreeMap<String, serde_json::Value>` so serialized component order is deterministic while values remain reflection-neutral. `WorldHierarchyRow` and `WorldInspectionFieldRow` are owned here and reused directly by `zircon_runtime` inspection; the editor does not reconstruct hierarchy anchors or field metadata from ad hoc JSON.

`generation_hint` is optional on all three projections. An exact hint match produces `NotModified { generation }`; a missing or stale hint returns `ComponentRows`, `HierarchyRows`, or `InspectionFields`, each with its generation. A focused entity that no longer exists returns `EntityMissing { generation, entity }` rather than an empty authoring fallback. Every materialized response establishes the generation anchor for the next query. The saturated `u64::MAX` revision always materializes because later monotonic mutations cannot produce a distinct counter value. Component rows are ordered by stable entity id; hierarchy rows preserve runtime inspection pre-order; Inspector fields preserve the runtime artifact's deterministic component/field order. No response carries editor selection or other authoring state.

The ABI producer enforces encoded-byte, item-count, nesting, and processing-time budgets before returning owned data. Hierarchy and focused Inspector queries validate borrowed inspection artifacts before cloning rows or fields. Snapshot IDs, cursors, cancellation, and multi-page recovery remain the open Interface02 P0 work; this generation-bearing typed projection does not claim million-entity paging or power targets. Focused Inspector cadence is an editor consumer policy, not part of this DTO module.

## Watch and invalidation model

`WatchKey` supports four planned invalidation families:

- a world subtree rooted at a runtime entity;
- a fully qualified component type;
- a stable `ResourceId` asset identity;
- any world-structure change.

Each registration receives an opaque `WatchToken`. Runtime stores and returns tokens, never editor `ViewInstanceId` values. The editor-side watch map introduced by M2 will own token-to-view projection and unregister tokens when a view closes.

`InvalidationBatch` carries one monotonic world generation, dirty tokens, and typed facts. Facts cover spawn/despawn/reparent, scene load/unload, and a count-only dynamic-scene asset reload report. The count DTO prevents the interface crate from importing runtime report implementation types or leaking scene reload internals across the ABI boundary.

## Wire rules and hard cut

Enums use stable snake-case `kind`/`data` serde tags. Structs and enum payloads reject unknown fields. Component filters/selectors may be empty, but the former untagged query object and bare `Rows(Vec<EntityRow>)` result are retired and not accepted. Retired field aliases, legacy view ids, fallback deserializers, and transport-specific wrappers are not accepted.

World entity identity is the runtime's current stable `u64` identity. Asset facts use the existing interface-owned `ResourceId`; no string path, editor tab id, or runtime object pointer is permitted in the protocol.

## Reference evidence

Bevy BRP supplies the primary precedent: `BrpQuery`, `BrpQueryFilter`, `BrpQueryRow`, and its serialization tests keep fully qualified component names and reflected values independent from the HTTP transport. Fyrox's centralized `on_sync_to_model` hook supplies the synchronization-point discipline, while Editor02 intentionally replaces its full-frame pull with generation hints and typed invalidation watches.

## Test coverage

`world_sync_contracts.rs` covers JSON round trips for component, hierarchy, and focused Inspector projections, every watch-key family, tokens, invalidation facts, and dynamic reload counts. It covers generation-bearing materialized results, explicit missing-entity state, matching/stale/missing generation hints, ascending component-row canonicalization, and rejection of retired wire fields. Runtime tests cover hierarchy/field artifact reuse and producer item-budget rejection. These Rust tests are updated in source but still require the coordinator-managed gate; static checks do not imply runtime acceptance.

## Follow-up

Editor02 M1.2 now owns authoritative runtime generation, split inspection reads, and stable subtree hashes in `zircon_runtime`; those behaviors remain outside this DTO crate. M2 owns runtime subscription matching and the editor invalidation pump. M3 owns binding dependencies and serialized session gateway transport. None of those behaviors may move into this interface DTO module.
