---
title: Runtime07 ECS Compiled Query Binding Current-Source Review
date: 2026-09-01
status: review_complete_model_profile_recorded_implementation_routed_runtime08_session_registration_blocked
origin_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
canonical_failure: docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-world-fixed-component-storage-and-stable-query-index.md
---

# Runtime07 ECS compiled query binding current-source review

## Decision

The current ECS query hot path is structurally wrong. `QueryState` compiles an archetype binding
plan, but every matching entity expands that plan into a fresh logical
`Vec<ComponentStorageLocation>`. Every typed data fetch and every `Added<T>` / `Changed<T>` filter
then linearly searches the same vector by `TypeId`. Tuple data and tuple filters repeat the scan.

The accepted direction is a hard cut to query-layout slots:

1. `QueryState<D, F>` compiles one typed layout for `D` and `F` after `QueryAccess` is complete.
2. Every typed leaf owns a `QueryComponentSlot`, and tuple layouts preserve those leaf slots.
3. Every `CachedArchetypePlan` stores a slot-aligned `Option<QueryComponentBinding>` array. Required
   bindings are present; optional bindings may be absent without changing later slot numbers.
4. Iterators pass a small compiled entity projection containing the selected plan and stable entity
   location. A typed fetch resolves its known slot directly and materializes only that location.
5. The old per-entity location vector, `rust_type_id` projection token, linear helpers, and duplicate
   cached-query filter/data implementations are deleted together. There is no compatibility lane.

This is the closest sound fit to Unreal Mass requirement mappings and Bevy query fetch state while
preserving Zircon's existing stable query-order contract. A per-filter cache or a second lookup map
beside the current vector is rejected because it would preserve two query execution models.

## Current-source evidence

The reviewed production path is:

```text
QueryState::update_cache
  -> compile_archetype_plan: Vec<QueryComponentBinding>
  -> stable/archetype iterator selects one entity
  -> CachedArchetypePlan::write_component_locations: visit K bindings
  -> QueryFilter tuple: linearly find TypeId for each Added/Changed leaf
  -> QueryData tuple: linearly find TypeId again for each fetched leaf
```

The duplicated work exists in both query families:

- `query_filter.rs::component_ticks_at_location<T>` performs `iter().find(...)` for each change
  filter.
- `query_data.rs::component_location<T>` repeats the same scan for ordinary cached data.
- `cached_query_iter.rs` owns another `component_location<T>` and
  `component_ticks_at_location<T>` implementation.
- `archetype_plan.rs::write_component_locations` clears, reserves, and pushes K locations per
  entity even though table column slots were already compiled per archetype.
- `query_iter.rs`, cached iterators, many iterators, combinations, and mutable iterators each retain
  a scratch location vector, so the architectural cost is cross-surface rather than one helper.

Current source hashes at review time:

| Path | SHA-256 |
| --- | --- |
| `scene/ecs/query/query_filter.rs` | `e4f6d27f3550288fcf0413f8589763e853a476f515ebde1dd35985abc051a305` |
| `scene/ecs/query/query_data.rs` | `713a8fb31845065bb43c83af37181b8f51b2ca48c65f9316fbc812c49381b88f` |
| `scene/ecs/query/cached_query_iter.rs` | `5998b50f67ce850a970be8a4b200b009dbc254d08e6589879d7db703328cb4bc` |
| `scene/ecs/query/query_state/archetype_plan.rs` | `d4b21b33e2a43bb0afdd75508f6098f4aa3cedcc28d0e1e362803ed661401d4e` |
| `scene/ecs/query/query_state/cache.rs` | `ab6e1da0c016c35488e44842c15c5939465e5398fbd441d0083461e3b49a8467` |
| `scene/ecs/query/query_iter.rs` | `6fc59c659208b778b25cf4d1d0c42b6693cddd49fe3c260742f184509e359b44` |
| `scene/ecs/storage/component_storage/location.rs` | `e8e7f9a40f2f9c8413738873f92fc5979b5f1d67bcc875d8c75243c5120c3db1` |

## Algorithm and deterministic work baseline

Let `E` be matching entities, `K` be unique read/filter components, `D` be typed data leaves, and
`F` be change-filter leaves. The current execution cost after cache compilation is:

```text
location projection = Theta(E * K)
typed lookup         = Theta(E * sum(position of every D/F leaf))
worst case           = Theta(E * K * (1 + D + F))
```

The first RED fixture must use four registered table components in deterministic slot order with
data leaves for all four and filters
`(Added<C>, Changed<C>, Added<D>, Changed<D>)`. On an all-matching initial window, current source
per entity performs four projection visits, ten data-location comparisons, and fourteen
filter-location comparisons: 28 binding/location visits. At 100,000 entities that is exactly
2,800,000 visits for one query pass. The target direct-slot path performs eight typed slot
resolutions and no K-wide location projection: 800,000 direct resolutions, independent of K's
position ordering.

The existing `ChangeDetectionScanStats::scanned_marks` semantic must remain four evaluated marks per
matching entity in this fixture. Binding/location work is a separate performance counter; it must
not inflate or reinterpret change-detection diagnostics.

The existing columnar acceptance test only reports one component and therefore cannot detect this
growth. It must be extended with:

- deterministic `binding_projection_visits`, `linear_location_probes`, and
  `direct_slot_resolutions` deltas;
- 1 / 1,000 / 100,000 entity scales;
- 1 / 4 / 8 unique component widths with multiple `Added` and `Changed` filters;
- release P50/P95 elapsed time after warmup;
- allocation count and requested bytes around iteration, with zero per-entity allocation growth;
- unchanged result order and unchanged added/changed match counts.

No elapsed-time or power improvement is claimed by this source review. Production mutation remains
gated on the RED work counter and a managed Windows release baseline bound to the exact source
manifest. Product power evidence remains a later real-engine capture, not a microbenchmark claim.

### Pre-implementation location-scan model

An isolated optimized Rust model on the E drive reproduced the current per-query scratch projection
and per-leaf linear location search against the accepted direct-slot lookup. It used 21 alternating
samples and cardinality-scaled iterations. Marker:
`RUNTIME07_ECS_QUERY_LOCATION_SCAN_MODEL_V1`.

| Width | Entities | Current projection visits | Current linear probes | Current P50 | Current P95 | Direct resolutions | Direct P50 | Direct P95 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 1 | 1 | 3 | 86 ns | 96 ns | 3 | 7 ns | 8 ns |
| 1 | 1,000 | 1,000 | 3,000 | 17.0 us | 17.9 us | 3,000 | 5.0 us | 5.6 us |
| 1 | 100,000 | 100,000 | 300,000 | 1.83 ms | 3.19 ms | 300,000 | 0.51 ms | 0.96 ms |
| 4 | 1 | 4 | 24 | 122 ns | 137 ns | 8 | 15 ns | 15 ns |
| 4 | 1,000 | 4,000 | 24,000 | 53.5 us | 76.7 us | 8,000 | 13.2 us | 20.9 us |
| 4 | 100,000 | 400,000 | 2,400,000 | 5.42 ms | 7.45 ms | 800,000 | 1.27 ms | 3.15 ms |
| 8 | 1 | 8 | 66 | 177 ns | 835 ns | 12 | 20 ns | 38 ns |
| 8 | 1,000 | 8,000 | 66,000 | 97.4 us | 165.2 us | 12,000 | 18.7 us | 177.9 us |
| 8 | 100,000 | 800,000 | 6,600,000 | 10.67 ms | 16.87 ms | 1,200,000 | 1.89 ms | 2.37 ms |

The current model allocates one reusable scratch vector per query pass; requested bytes are 16, 64,
and 128 for widths 1, 4, and 8 in the model representation. The direct-slot pass allocates zero.
Those byte totals describe the model's 16-byte location record and are not a claim about the
current-source `ComponentStorageLocation` layout. The work counts are exact for the modeled query
shape: width four yields `400,000 + 2,400,000 = 2,800,000` visits at 100,000 entities, matching the
deterministic RED derivation above.

At 100,000 entities the direct model improves P50 by 3.58x, 4.28x, and 5.64x for widths 1, 4, and 8.
The small-cardinality P95 samples show Windows scheduling noise, so they are not acceptance ratios.
The model proves direction and scale only; a managed current-source benchmark must still measure
real table/SparseSet access, result parity, change-tick statistics, allocations, and elapsed P50/P95.

## Reference-engine review

### Unreal Mass

`FMassEntityQuery::CacheArchetypes` in
`dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassEntityQuery.cpp` incrementally caches
matching archetypes and builds `ArchetypeFragmentMapping` once for each new archetype. The mapping
is aligned with the sorted fragment requirements.

`FMassArchetypeData::BindEntityRequirements` in
`dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassArchetypeData.cpp` consumes those
indices by requirement position and binds fragment array views for a chunk. Entity work indexes an
already-bound view; it does not search a type list for every fragment and entity. The slow lookup is
kept only as an explicit no-mapping fallback, which Zircon must not reproduce as a second lane.

### Bevy ECS

`dev/bevy/crates/bevy_ecs/src/query/fetch.rs` separates persistent `WorldQuery::State` from runtime
`Fetch`. `set_table` / `set_archetype` bind the current storage before `fetch` is called for rows.
Change-detection fetches carry their already-selected tick storage rather than searching a vector
of type-tagged locations per entity.

The common rule is compile requirement identity once, bind storage when the archetype/table changes,
then index directly during entity iteration.

## Required hard-cut scope

The minimum coherent source slice is larger than `query_filter.rs`:

- query layout state: `query_data.rs`, `query_filter.rs`, and their tuple implementations;
- archetype binding plan and compiler: `query_state/archetype_plan.rs`, `query_state/cache.rs`, and
  `query_state/state.rs`;
- all read, mutable, many, unique-many, combination, and direct cached iterators;
- removal of the duplicate `CachedQueryData` / `CachedQueryFilter` lookup implementation or its
  convergence on the same compiled projection;
- removal of `ComponentStorageLocation::rust_type_id` and `with_rust_type_id` after all typed
  location scans are gone;
- structural tests that currently require the `rust_type_id` per-row projection must be replaced
  with guards requiring slot-aligned plans and forbidding `iter().find` location scans.

Required behavioral coverage includes table and SparseSet components, optional data, read-only and
mutable data, stable iteration order, many/combinations, missing optional bindings, archetype
addition, entity moves, tick wraparound, and unchanged `ChangeDetectionScanStats` deltas.

## Ownership and execution state

No ECS production or test file was edited during this review. The existing canonical Runtime08
failure remains the sole handoff record; no duplicate failure was created.

The intended coordinator session
`root-runtime08-query-binding-r1-20260901` was submitted twice with the Runtime08 plan and exact
query/test write scope. Both registration calls timed out, and two recovery reads returned
`Unknown Session`. Therefore the implementation is not legally owned and is not started. Runtime07
continues other owned non-validation work rather than using the validation queue or an ownership
workaround as its only activity.

## Acceptance checklist

- [x] Reviewed every current query-plan, filter, data, iterator, storage-location, diagnostic, and
  existing performance-fixture boundary relevant to the scan.
- [x] Compared the whole binding model with local Unreal Mass and Bevy ECS sources.
- [x] Defined the exact complexity and a deterministic 100,000-entity work-count RED.
- [x] Recorded a 1/1k/100k by 1/4/8 width allocation and P50/P95 pre-implementation model.
- [x] Defined the only accepted slot-aligned hard-cut architecture and forbidden fallback.
- [ ] Runtime08 coordinator ownership established.
- [ ] RED counter and current-source managed Windows release baseline recorded.
- [ ] Production query projection hard-cut implemented.
- [ ] Focused behavior, 1/1k/100k width matrix, allocation, P50/P95, and source guards accepted.
- [ ] Terminal receipt attributed and returned to Runtime02 source task.
