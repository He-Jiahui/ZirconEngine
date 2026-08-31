# Runtime134 Lighting Model Zero-Allocation Format

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime134-editor80-performance-batch-20260826cq-v1`

## Problem

`RenderMaterialLightingModel::fmt` formatted every model through `as_token()`. Built-in models
therefore allocated a temporary `String` even though their wire tokens are static. Serde followed
the same allocating path for the three built-in variants.

## Optimization

- Centralize built-in model tokens behind a private static-token projection.
- Write built-in tokens directly to `Formatter` and `Serializer` without an intermediate owner.
- Preserve `as_token()`, custom model formatting, parsing aliases, and serialized values.

## Regression Contract

The shared `optimization_batch_20260826cq_` filter owns three Runtime tests: token behavior, source
shape, and an ignored paired release P95 benchmark. The benchmark emits
`RUNTIME134_LIGHTING_MODEL_ZERO_ALLOCATION_FORMAT_BENCH_V1`, compares the former allocating
format path with direct formatting across 90,000 built-in formats per sample, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
