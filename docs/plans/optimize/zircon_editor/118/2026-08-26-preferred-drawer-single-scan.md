# Editor118 Preferred Drawer Single Scan

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime172-editor118-performance-batch-20260826ec-v1`

## Problem

Tab-drag host resolution searched the same drawer-slot candidates up to four times: active view,
active tab, populated stack, then visible drawer. Each pass repeated ordered-map lookup and drawer
state checks.

## Optimization

- Classify each candidate drawer once and preserve the four existing global priority tiers.
- Retain the first slot within each tier and return immediately for the first active-view drawer.
- Preserve the configured group fallback when no visible candidate exists.

## Regression Contract

The shared `optimization_batch_20260826ec_` filter owns three Editor tests: priority/order
semantics, single-loop source shape, and an ignored paired release P50/P95 benchmark. The benchmark
emits `EDITOR118_PREFERRED_DRAWER_SINGLE_SCAN_BENCH_V1`, scans 256 candidates 8,192 times per
sample, reduces traversal passes from four to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
