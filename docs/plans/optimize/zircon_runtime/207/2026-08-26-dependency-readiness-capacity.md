# Runtime207 Dependency Readiness Capacity

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime207-editor153-performance-batch-20260826fl-v1`

## Problem

Asset dependency readiness traversal grew its result rows, row index, expanded set, and breadth
first queue from empty even though the complete direct dependency count was already available.

## Optimization

- Use the direct dependency count as the initial capacity for the result rows, discovered set, and
  traversal queue.
- Deduplicate dependency ids at enqueue time, replacing the old row-index plus expanded-set pair;
  preserve breadth-first order, direct/depth precedence, transitive expansion, missing-row
  diagnostics, and root cycle protection.

## Regression Contract

The `optimization_batch_20260826fl_` Runtime tests traverse 256 real missing asset ids against a
default readiness generation, verify row order, direct/depth flags, failed state, diagnostics and
capacity, enforce the three current production reservations, and compare the optimized traversal
against the legacy graph on cyclic/shared/missing fixtures. An ignored paired release benchmark
still emits `RUNTIME207_DEPENDENCY_READINESS_CAPACITY_BENCH_V1`; its synthetic container model is
kept as a supplementary allocation signal and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
