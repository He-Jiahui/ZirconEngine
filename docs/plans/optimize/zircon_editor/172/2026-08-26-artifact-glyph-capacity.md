# Editor172 Artifact Glyph Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime226-editor172-performance-batch-20260826ge-v1`

## Problem

Retained text artifact projection grew its glyph geometry vector from empty even though validated
artifact lines expose every source glyph count before geometry projection starts.

## Optimization

- After shared-layout and raster-face validation, preflight every line's glyph slice and use
  checked addition to derive a safe glyph capacity upper bound.
- Allocate the glyph geometry vector once from that upper bound while retaining rasterization and
  rotation filters during projection.
- Leave font-index and raster-font collections demand-grown because their unique face count is not
  represented by the glyph count.
- Preserve whole-layout fallback, glyph order and geometry, raster face conversion, font indices,
  smoothing, and text placement.

## Regression Contract

The `optimization_batch_20260826ge_` Editor tests cover checked capacity accumulation and enforce
preflight-before-allocation ordering, and provide an ignored paired release benchmark emitting
`EDITOR172_ARTIFACT_GLYPH_CAPACITY_BENCH_V1`. It projects 32 layouts of 16,384 glyphs per sample,
including the optimized preflight cost, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
