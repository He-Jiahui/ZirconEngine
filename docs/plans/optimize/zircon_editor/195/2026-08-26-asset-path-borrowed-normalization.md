# Editor195 Asset Path Borrowed Normalization

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime249-editor195-performance-batch-20260826hc-v1`

## Problem

Visual-asset candidate normalization always allocated a full replacement string even when the input
already used forward slashes. Inputs beginning with `res://` then allocated and copied the stripped
suffix into a second complete string before building the relative path.

## Optimization

- Borrow trimmed forward-slash inputs without materializing a replacement string.
- Allocate a normalized string only when the input actually contains backslashes.
- Strip `res://` as a borrowed slice for both paths before retaining accepted components.

## Regression Contract

The `optimization_batch_20260826hc_` Editor tests preserve resource prefixes, leading `assets`,
Windows separators, and ignored parent components; enforce borrowed/owned normalization branches;
and provide an ignored paired release benchmark emitting
`EDITOR195_ASSET_PATH_BORROWED_NORMALIZATION_BENCH_V1`. It repeatedly normalizes a long forward-slash
resource path and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
