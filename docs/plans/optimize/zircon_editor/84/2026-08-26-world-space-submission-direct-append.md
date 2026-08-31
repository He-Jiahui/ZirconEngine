# Editor84 World-Space Submission Direct Append

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime138-editor84-performance-batch-20260826cu-v1`

## Problem

World-space UI scene collection created a temporary `Vec` and sorted it for every pane node model
and floating-window header, then immediately extended the final scene vector and sorted the whole
scene again. A dock pane can visit ten node models, so this repeated allocation and local sorting
on every scene projection.

## Optimization

- Add a caller-owned append path that streams eligible node submissions into the final scene
  vector.
- Route all pane node models and floating-window headers through that append path.
- Retain the standalone builder and its local sort for existing independent callers and tests.
- Preserve the final scene comparator across render order, surface, node, and control identifiers.

## Regression Contract

The shared `optimization_batch_20260826cu_` filter owns three Editor tests: append behavior with an
existing prefix, standalone builder ordering, and an ignored paired release P50/P95 benchmark. The
benchmark emits `EDITOR84_WORLD_SPACE_SUBMISSION_DIRECT_APPEND_BENCH_V1`, collects 32 groups of 96
world-space nodes, records the reduction from 32 temporary vectors/local sorts to zero, and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
