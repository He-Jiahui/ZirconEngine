---
title: Runtime08D Borrowed Polygon Index Projection
category: zircon_runtime
report_id: Runtime08D-borrowed-polygon-indices-2026-08-26
date: 2026-08-26
session_id: root-runtime08d-borrowed-polygon-indices-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime08D Borrowed Polygon Index Projection

## Scope

This slice removes one temporary `Vec<u32>` from every built-in baked-navmesh polygon projection.
It does not change polygon index range clamping, triangle grouping, canonical undirected edge keys,
edge sorting/deduplication, vertex lookup, vertex sorting/deduplication, bounds, center, area,
adjacency, spatial indexing, query behavior, or public APIs.

Adjacent navigation runtime owner files contain unrelated active work and are deliberately outside
this candidate. The changed `baked_mesh.rs` file and its topology tests were clean at lease claim.

## Change

- `BakedPolygon::from_asset` now borrows the validated range directly from `asset.indices`.
- `polygon_edge_keys` already accepted `&[u32]`; it now receives that slice without an extra
  reference layer.
- The vertex projection iterates the same borrowed slice, so edge and vertex construction share one
  immutable source without copying indices.
- Existing tests retain the canonical edge-key, shared-edge adjacency, and two-triangle mesh
  behavior oracles. A Python source contract prevents `to_vec()` from returning to this path.

## Deterministic Performance Evidence

The independent release model projects 32,768 polygons with two triangles and six indices each.
It retains the real edge-key and vertex output allocations in both variants; only the temporary
index copy differs. Each run uses 21 paired samples with alternating baseline/optimized order.

| Evidence | Owned index copy | Borrowed index slice | Result |
| --- | ---: | ---: | ---: |
| Projection checksum | 294,912 | 294,912 | identical |
| Total allocations | 131,072 | 98,304 | 32,768 fewer; 25% reduction |
| Run 1 P50 | 13.628 ms | 10.866 ms | 20.269% faster |
| Run 1 P95 | 31.081 ms | 17.581 ms | 43.436% faster |
| Run 2 P50 | 12.289 ms | 10.366 ms | 15.641% faster |
| Run 2 P95 | 15.123 ms | 12.271 ms | 18.861% faster |
| Run 3 P50 | 12.741 ms | 10.727 ms | 15.806% faster |
| Run 3 P95 | 14.228 ms | 14.347 ms | 0.839% slower |

The managed gate requires the exact checksum, exact allocation counts and 32,768-allocation delta,
at least 25% total allocation reduction, at least 10% P50 improvement, and optimized P95 no more
than 15% above baseline. The third run records the observed scheduler-sensitive P95 variance.

## Acceptance

- TDD RED observed two owned-index contract failures while all three existing topology oracle names
  remained present.
- `tools.tests.test_runtime08d_borrowed_polygon_indices_performance_contract` passes 3/3 locally.
- Exact production/model `rustfmt --check`, Python compilation, PowerShell parsing, three paired
  model runs, and scoped diff checks pass locally.
- The baked-mesh performance-contract Rust test module, source contracts, formatting, performance
  model, and scoped diff checks are submitted together in one coordinator validation ticket with
  one Cargo command.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Runtime08D still owns the unique navigation authority, prepared persistent Detour owner, real bake
geometry/config, tile lifecycle, bounded bake tasks, incremental world projection, movement and AI
contracts, bounded debug overlay, Editor bake workflow, and product-scale qualification.
