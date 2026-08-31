# Runtime86 Terrain Reference Append Optimization Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: `docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md`, G21/G22
- Status: implementation and release gate authored; batched managed validation pending

## Problem

`TerrainAsset` and `TerrainLayerStackAsset` used `flat_map` over
`TerrainLayerAsset::direct_references`. Each layer first allocated a temporary
zero-to-two-entry `Vec<AssetReference>`, then the outer collector copied those
references into the final dependency vector. Large terrain documents therefore
performed one short-lived heap allocation per layer.

## Change

- Add one layer-owned `direct_reference_count` authority for the two optional
  reference fields.
- Add an internal append path that clones references directly into the caller's
  final vector.
- Reserve the exact total reference count once for both terrain and layer-stack
  extraction.
- Keep the public layer-level `direct_references()` API, material-before-weightmap
  order, layer order, output type, and reference clone semantics unchanged.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| 4,096 layers, two references per layer | 4,096 temporary layer vectors | 0 temporary layer vectors | 100% removed |
| Final dependency vector | progressively collected | exact capacity 8,192 | growth removed for the measured workload |
| Reference order | material then weightmap per layer | material then weightmap per layer | unchanged |

The ignored release gate runs 17 alternating legacy/append sample pairs over
4,096 layers and 8,192 references. Acceptance requires append nearest-rank P95
to be at most 75% of legacy P95, a minimum 25% reduction. Exact Windows timing
values remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826c_runtime86_terrain_references_append_into_one_reserved_buffer`
  locks exact capacity and the append production shape.
- `optimization_batch_20260826c_runtime86_terrain_references_preserve_layer_order`
  covers material-only, weightmap-only, and dual-reference layers.
- `optimization_batch_20260826c_runtime86_terrain_reference_append_performance_evidence`
  emits `RUNTIME86_TERRAIN_REFERENCE_APPEND_BENCH_V1`, raw samples, layer and
  reference counts, temporary-vector counts, and the 25% P95 threshold.
- Exact-file Rust 1.94.1 rustfmt, source contracts, and scoped diff checks must
  pass before managed validation submission.

## Remaining Plan Work

This slice does not close Runtime86. The canonical asset-type catalog, complete
dependency coverage manifest, typed dependency graph generation, schema/codec
versioning, plugin types, and cross-product scale gates remain open.
