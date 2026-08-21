# Plugins14 Arc Loaded-Navmesh Snapshot Optimization Record

- Date: 2026-08-19
- Owner: `plugins14-arc-assets-demand-overlay-r1-01a00797-20260819`
- Source plan: `docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md`, NNAV-P1-029 and NNAV-P1-035
- Status: implementation complete; combined managed validation pending

## Problem

Every navigation agent tick called `loaded_assets()`, cloned every complete
`NavMeshAsset` out of the manager mutex, collected the clones into a vector,
and sorted it by handle. Large vertex, index, polygon, tile, and link buffers
were therefore copied once per frame even though loaded assets are immutable
between explicit publication operations.

## Change

- Loaded assets are retained as `Arc<NavMeshAsset>` in a `BTreeMap` keyed by
  the raw stable handle value.
- Agent tick snapshots clone only the Arc handles and inherit deterministic
  numeric-handle order directly from the map.
- Public query paths that promise owned `NavMeshAsset` values still clone at
  that explicit boundary; no mutable asset reference is exposed.
- Load, generated-bake replacement, clearing, default-handle selection,
  overlay generation, and statistics preserve their previous semantics.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Snapshot 16 assets x 8,192 triangles | 16 deep asset clones | 16 Arc clones | 100% of mesh-buffer copies |
| Polygon records copied per snapshot | 131,072 | 0 | 100% |
| Handle ordering | collect + sort | ordered map traversal | 1 sort to 0 |

## Acceptance

- `loaded_asset_snapshots_share_the_immutable_navmesh_allocation` proves two
  snapshots point at the same immutable asset allocation.
- Existing manager, crowd, operation, and overlay tests retain the public
  loaded-asset and default-handle behavior.
- `arc_loaded_navmesh_snapshot_release_benchmark_evidence` compares 21 paired,
  alternating release samples over 16 assets of 8,192 triangles and four
  snapshots, then computes nearest-rank P50/P95.
- Timing gate: Arc snapshot P95 must be no more than 20% of legacy deep-clone
  P95.
- Exact-file Rustfmt, scoped source assertions, and `git diff --check`: passed.
- Cargo regression and release P50/P95: pending one batched Windows
  coordinator validation with demand-driven overlay projection.

## Remaining Scope

Each synchronous query still clones one selected asset before the native or
fallback backend builds its query state. Persistent per-World nav generations,
native query pooling, explicit unload/streaming, and generation leases remain
required to close NNAV-P1-029/035 rather than only fixing agent-tick snapshots.
