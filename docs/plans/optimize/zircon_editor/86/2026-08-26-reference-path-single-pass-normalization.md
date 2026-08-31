# Editor86 Reference Path Single-Pass Normalization

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime140-editor86-performance-batch-20260826cw-v1`

## Problem

Five workbench reference-builder paths normalized labels with `replace(' ', "_")` followed by
`to_ascii_lowercase()`. Each call allocated an intermediate string that was immediately discarded,
and these paths are regenerated whenever the reference workbench surface is built.

## Optimization

- Add one capacity-reserved path-segment normalizer owned by the reference builder module.
- Translate spaces and ASCII uppercase characters during a single character pass.
- Route status, viewport toolbar, inspector, input sample, and list sample path construction through
  the helper while leaving unrelated lowercase-only paths unchanged.
- Preserve repeated spaces, punctuation, non-ASCII characters, and existing path output.

## Regression Contract

The shared `optimization_batch_20260826cw_` filter owns three Editor tests: legacy-output parity,
five-call-site/source contracts, and an ignored paired release P50/P95 benchmark. The benchmark
emits `EDITOR86_REFERENCE_PATH_SINGLE_PASS_NORMALIZATION_BENCH_V1`, normalizes 8,192 labels,
records the per-label allocation reduction from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
