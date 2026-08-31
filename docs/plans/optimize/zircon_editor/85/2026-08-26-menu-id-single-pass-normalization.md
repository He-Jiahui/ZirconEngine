# Editor85 Menu ID Single-Pass Normalization

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime139-editor85-performance-batch-20260826cv-v1`

## Problem

Each top-level menu reflected into the workbench model first allocated an ASCII-lowercase label and
then allocated a second string to replace spaces with underscores. The first allocation was
immediately discarded.

## Optimization

- Reserve one output string using the source label byte length.
- Lowercase ASCII characters and translate spaces to underscores during the same character pass.
- Preserve non-ASCII characters, repeated spaces, existing underscores, and all call-site output
  ownership.

## Regression Contract

The shared `optimization_batch_20260826cv_` filter owns three Editor tests: legacy-output parity,
single-buffer source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR85_MENU_ID_SINGLE_PASS_NORMALIZATION_BENCH_V1`, normalizes 8,192 representative menu labels,
records the per-label allocation reduction from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
