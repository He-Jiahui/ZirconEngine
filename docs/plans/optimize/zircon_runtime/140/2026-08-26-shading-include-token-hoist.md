# Runtime140 Shading Include Token Hoist

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime140-editor86-performance-batch-20260826cw-v1`

## Problem

Resolving one plugin shading-model include normalized the same include token for every ready shader
record. Each locator comparison also allocated a formatted `/{token}` suffix. A project with many
shader records therefore repeated token and suffix allocations throughout each include scan.

## Optimization

- Normalize the requested include token once before iterating shader records.
- Pass the normalized token through primary and artifact locator matching.
- Replace formatted suffix construction with `strip_suffix` plus an explicit slash-boundary check.
- Preserve path trimming, slash normalization, ASCII case folding, `.wgsl` stripping, duplicate
  detection, and original-token diagnostics/output.

## Regression Contract

The shared `optimization_batch_20260826cw_` filter owns three Runtime tests: locator-shape parity,
source-level hoist/suffix contracts, and an ignored paired release P50/P95 benchmark. The benchmark
emits `RUNTIME140_SHADING_INCLUDE_TOKEN_HOIST_BENCH_V1`, scans 8,192 shader paths, records token
normalizations from 8,192 to one and suffix allocations from 8,192 to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
