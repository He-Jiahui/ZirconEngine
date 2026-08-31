# Runtime151 Extension Token In-Place Normalization

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime151-editor97-performance-batch-20260826dh-v1`

## Problem

Geometry-source and shading-model registration allocated a trimmed lowercase token, cloned it back
into the owned descriptor, and retained the first allocation as the registry key. Registration
therefore required two new token buffers even though the descriptor token was already owned.

## Optimization

- Validate the `custom:<name>` prefix against borrowed trimmed text before mutation.
- Trim and ASCII-lowercase the descriptor's existing token buffer in place.
- Clone only the required registry key, preserving normalized descriptor metadata and all later
  validation diagnostics.

## Regression Contract

The shared `optimization_batch_20260826dh_` filter owns three Runtime tests: canonical/error
behavior, descriptor-buffer reuse plus caller source shape, and an ignored paired release P50/P95
benchmark. The benchmark emits `RUNTIME151_EXTENSION_TOKEN_IN_PLACE_NORMALIZATION_BENCH_V1`,
normalizes 16,384 tokens per sample, records normalization allocations from 16,384 to zero, and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
