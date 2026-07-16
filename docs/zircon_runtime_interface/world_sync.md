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

- `query.rs` owns `WorldQuery`, component presence filters and selectors, deterministic entity rows, and `WorldQueryResult`.
- `watch.rs` owns typed `WatchKey` values, registrations, and runtime-issued opaque `WatchToken` identities.
- `invalidation.rs` owns `WorldFact`, the stable dynamic-scene reload summary, and `InvalidationBatch`.
- `mod.rs` is a navigational façade and contains no query, matching, or serialization behavior.

This four-file shape follows Editor02 M1.1 and keeps future subscription-table or gateway implementations out of the interface contract.

## Query model

`QueryFilter.with` and `QueryFilter.without` contain fully qualified component type names. `WorldQuery.select` lists component values to project for each matching entity. `EntityRow` uses `BTreeMap<String, serde_json::Value>` so serialized row order is deterministic while component payloads remain reflection-neutral.

`generation_hint` is optional. `WorldQuery::result_for_generation` is the single contract helper for the short circuit: an exact hint match produces `NotModified { generation }`; a missing or stale hint returns `Rows`. The saturated `u64::MAX` revision always returns `Rows`, because later monotonic mutations cannot produce a distinct counter value. The helper canonicalizes returned rows by ascending stable entity id, while each row's component map is already key ordered. Rows deliberately do not carry editor selection, focus, hierarchy presentation, or any other authoring state.

## Watch and invalidation model

`WatchKey` supports four planned invalidation families:

- a world subtree rooted at a runtime entity;
- a fully qualified component type;
- a stable `ResourceId` asset identity;
- any world-structure change.

Each registration receives an opaque `WatchToken`. Runtime stores and returns tokens, never editor `ViewInstanceId` values. The editor-side watch map introduced by M2 will own token-to-view projection and unregister tokens when a view closes.

`InvalidationBatch` carries one monotonic world generation, dirty tokens, and typed facts. Facts cover spawn/despawn/reparent, scene load/unload, and a count-only dynamic-scene asset reload report. The count DTO prevents the interface crate from importing runtime report implementation types or leaking scene reload internals across the ABI boundary.

## Wire rules and hard cut

Enums use stable snake-case `kind`/`data` serde tags. Structs and enum payloads reject unknown fields. Optional and empty query collections have intentional defaults, matching Bevy BRP query behavior; retired field aliases, legacy view ids, fallback deserializers, and transport-specific wrappers are not accepted.

World entity identity is the runtime's current stable `u64` identity. Asset facts use the existing interface-owned `ResourceId`; no string path, editor tab id, or runtime object pointer is permitted in the protocol.

## Reference evidence

Bevy BRP supplies the primary precedent: `BrpQuery`, `BrpQueryFilter`, `BrpQueryRow`, and its serialization tests keep fully qualified component names and reflected values independent from the HTTP transport. Fyrox's centralized `on_sync_to_model` hook supplies the synchronization-point discipline, while Editor02 intentionally replaces its full-frame pull with generation hints and typed invalidation watches.

## Test coverage

`world_sync_contracts.rs` covers JSON round trips for queries, every watch-key family, tokens, invalidation facts, and dynamic reload counts. It directly covers matching/stale/missing generation hints, ascending entity-row canonicalization, and rejection of unknown retired wire fields. The coordinator-managed M1 interface gate passed on Windows; the full M1 runtime acceptance gate remains tracked by the Editor02 child plan and is not implied by this crate-local result.

## Follow-up

Editor02 M1.2 now owns authoritative runtime generation, split inspection reads, and stable subtree hashes in `zircon_runtime`; those behaviors remain outside this DTO crate. M2 owns runtime subscription matching and the editor invalidation pump. M3 owns binding dependencies and serialized session gateway transport. None of those behaviors may move into this interface DTO module.
