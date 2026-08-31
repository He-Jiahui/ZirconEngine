# Runtime256 Streaming Material References

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime256-editor202-performance-batch-20260826hj-v1`

## Problem

Material dependency projection cloned the complete texture-reference vector into an intermediate
allocation before moving those values into an already capacity-sized result. Every dependency
query therefore allocated and released a redundant vector buffer.

## Optimization

- Clone texture references directly from the borrowed slice into the final result.
- Keep the shader-first ordering and the exact result capacity contract.
- Preserve independent owned `AssetReference` values in the returned vector.

## Regression Contract

The `optimization_batch_20260826hj_` Runtime tests preserve shader-first ordering and texture
deduplication; enforce slice-to-destination cloning without a whole-vector clone; and provide an
ignored paired release benchmark emitting `RUNTIME256_STREAMING_MATERIAL_REFERENCE_BENCH_V1`. It
extends 16,384 values 128 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
