# Editor205 Direct Arc Image Opacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime259-editor205-performance-batch-20260826hm-v1`

## Problem

Retained-host image opacity copied every RGBA payload into a temporary vector, modified alpha in
that vector, and then converted the vector into an `Arc<[u8]>`. The conversion required another
full shared-slice allocation and copy on every painted image.

## Optimization

- Copy the borrowed RGBA slice directly into its final `Arc<[u8]>` allocation.
- Reuse the existing shared pixel storage without copying when opacity is fully opaque.
- Mutate the uniquely owned Arc storage in place before publishing it.
- Preserve RGB bytes, opacity clamping, alpha rounding, and source immutability.

## Regression Contract

The `optimization_batch_20260826hm_` Editor tests preserve pixel and alpha results, source reuse,
and opaque Arc identity; enforce zero-copy opaque reuse plus direct Arc allocation for translucent
images; and provide an ignored paired release benchmark emitting
`EDITOR205_DIRECT_ARC_IMAGE_OPACITY_BENCH_V1`. It processes 16,384 opaque RGBA pixels 128 times per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
