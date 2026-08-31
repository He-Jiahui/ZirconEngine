# Runtime169 Collapse Borrowed Attributes

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime169-editor115-performance-batch-20260826dz-v1`

## Problem

MUI Collapse class generation cloned orientation, transition status, and collapsed-size strings for
formatting or predicates. Slot orientation repeated the same temporary allocation.

## Optimization

- Borrow and trim Collapse string attributes through one local helper.
- Return static orientation and status defaults without allocation.
- Apply the helper to component, hidden-state, and owner-slot paths.

## Regression Contract

The shared `optimization_batch_20260826dz_` filter owns three Runtime tests: defaults/hidden
behavior, borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The
benchmark emits `RUNTIME169_COLLAPSE_BORROWED_ATTRIBUTES_BENCH_V1`, performs 524,288 lookups per
sample, reduces lookup allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
