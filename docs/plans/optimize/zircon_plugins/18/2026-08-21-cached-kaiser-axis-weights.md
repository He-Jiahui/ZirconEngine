# Plugins18 Cached Kaiser Axis Weights Optimization Record

- Date: 2026-08-21
- Owner: `optimize-plugins18-kaiser-axis-weights-r1-01a00797-20260821`
- Source plan: `docs/plans/optimize/zircon_plugins/18-first-party-texture-source-importer-runtime-editor-dist-catalog-image-cubemap-array-volume-compression-streaming-product-integration-review.md`, TEX-P1-026 and TEX-P1-027
- Status: implementation complete; combined managed validation pending

## Problem

The current mip kernel attempted to hoist the Kaiser normalizer but routed it to
the Box helper and referenced it from the Kaiser helper without a parameter.
That source could not compile. A direct repair that recomputed Kaiser window
weights inside every output pixel would also repeat the same X-axis weights for
every output row and the same Y-axis weights for every output column.

## Change

- The invalid Box/Kaiser parameter routing is removed. Box and normal-map mips
  no longer prepare any Kaiser state.
- Kaiser color mips normalize once per generated level and cache at most five
  `(source coordinate, weight)` samples for every target X and Y coordinate.
- Each target texel reuses the cached separable axis weights while preserving
  the original sample iteration, weighted accumulation, fallback, color-space,
  alpha, and UNORM8 encoding order.
- A 7x5 sRGB regression compares the cached path byte-for-byte with the inline
  reference. The existing footprint regression continues to compare Box and
  Kaiser behavior.

## Deterministic Performance Evidence

The managed release gate downsamples the same deterministic 256x256 linear
RGBA8 source to 128x128 with the Kaiser filter. The inline and cached branches
produce byte-identical output and evaluate the same filter.

| Measure | Inline reference | Cached axes | Gate |
|---|---:|---:|---:|
| Target texels | 16,384 | 16,384 | exact |
| Kaiser weight evaluations | 487,305 | 1,274 | 99.739% eliminated |
| Kaiser normalizer evaluations | 1 | 1 | exact |
| Timing distribution | 21 samples | 21 samples | alternating first-run order |
| Nearest-rank P95 | pending | pending | cached <= 25% of inline |

Exact Windows P50/P95 values remain pending the combined coordinator batch and
must be written here before integration acceptance.

The pinned Plugins18 child validator is
`zircon-validation-plugins18-kaiser-axis-cache.ps1` at SHA-256
`E11EF2CC0A0537C1ADDADAC63C16721388A394B9847DAEF6A89ADB6911123609`.
It is aggregated with the existing eight plugin batches by
`zircon-validation-plugin-super-batch-nine.ps1` at SHA-256
`AECCAA0759853435CB662BB28D8C68A6F037F9E49DF09E7DABD26290B92F73F0`.

## Acceptance

- The repaired kernel has one defined Kaiser normalizer owner and no Kaiser
  argument on the Box helper.
- Cached and inline Kaiser output is byte-identical for an odd 7x5 sRGB source
  and for the 256x256 linear release workload.
- The full `mipgen` test filter runs in the behavior Cargo group, so the compile
  repair, offline chain construction, Box, normal-map, and Kaiser regressions
  share one result.
- `cached_kaiser_axis_weights_release_benchmark` emits 21 alternating raw sample
  pairs, recomputable nearest-rank P50/P95 values, and exact weight-evaluation
  counts.
- Exact-file Rustfmt, scoped diff checks, Cargo regressions, and release timing
  are required in one managed multi-task Windows validation copy. No per-task
  Cargo invocation is used.

## Remaining Scope

This slice keeps the current RGBA8 D2/Cube algorithm. It does not add float/HDR
mip generation, 3D filtering, sRGB alpha-coverage policy, normal roughness
variance, semantic filter selection, or a production GPU backend. TEX-P1-026
and TEX-P1-027 remain open beyond this compile repair and axis-cache hot path.
