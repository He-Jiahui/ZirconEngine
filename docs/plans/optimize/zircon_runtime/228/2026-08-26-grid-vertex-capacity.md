# Runtime228 Grid Vertex Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime228-editor174-performance-batch-20260826gh-v1`

## Problem

Runtime scene-grid construction grew its vertex vector from empty even though the fixed half extent
determines the exact number of line vertices before construction begins.

## Optimization

- Name the grid half extent, index count, and vertices-per-index contract and derive exact capacity
  from those constants.
- Reuse the half-extent constant for the iteration range and line endpoints so capacity and geometry
  cannot drift apart.
- Preserve all 21 grid indices, four vertices per index, axis/major/minor colors, order, and world
  coordinates while avoiding repeated vector growth.

## Regression Contract

The `optimization_batch_20260826gh_` Runtime tests cover the exact 84-vertex output and enforce the
extent-derived capacity contract, and provide an ignored paired release benchmark emitting
`RUNTIME228_GRID_VERTEX_CAPACITY_BENCH_V1`. It builds 32,768 grids per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
