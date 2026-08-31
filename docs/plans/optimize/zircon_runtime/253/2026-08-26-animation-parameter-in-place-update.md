# Runtime253 Animation Parameter In-Place Update

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime253-editor199-performance-batch-20260826hg-v1`

## Problem

Every finite animation parameter update converted the borrowed parameter name into a new `String`
before inserting it into the `BTreeMap`. Parameters are normally updated by stable names every
frame, so the existing-key path allocated and discarded a key on every value change.

## Optimization

- Preserve rejection of non-finite parameter values before any mutation.
- Update an existing parameter value through `get_mut` without allocating its key.
- Allocate an owned key only when a new parameter is introduced.

## Regression Contract

The `optimization_batch_20260826hg_` Runtime tests preserve existing and new parameter updates plus
non-finite rejection; enforce the in-place existing-key branch; and provide an ignored paired
release benchmark emitting `RUNTIME253_ANIMATION_PARAMETER_IN_PLACE_BENCH_V1`. It repeatedly updates
an existing parameter with a 32 KiB name and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
