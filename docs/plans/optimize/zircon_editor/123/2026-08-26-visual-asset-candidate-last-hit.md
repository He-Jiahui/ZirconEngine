# Editor123 Visual Asset Candidate Last Hit

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime177-editor123-performance-batch-20260826eh-v1`

## Problem

Retained-host visual asset candidate construction scanned the complete candidate vector for every
deduplication request, even when the new path repeated the just-appended final candidate. Variant
and alias expansion frequently generates such adjacent duplicates.

## Optimization

- Compare the final candidate before starting the full duplicate scan.
- Return immediately for adjacent duplicate paths.
- Preserve empty-path handling, SVG extension ordering, and full non-adjacent deduplication.

## Regression Contract

The shared `optimization_batch_20260826eh_` filter owns three Editor tests: variant/order behavior,
last-before-scan source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR123_VISUAL_ASSET_CANDIDATE_LAST_HIT_BENCH_V1`, performs 8,192 last-candidate pushes per
sample over 256 long paths, reduces path comparisons per last hit from 256 to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
