# Runtime170 Surface Child Borrowed Attributes

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime170-editor116-performance-batch-20260826ea-v1`

## Problem

MUI surface-child class generation cloned six position, placement, orientation, Drawer, and icon
attributes before formatting or predicates. None of these temporary strings escaped their owner.

## Optimization

- Borrow and trim surface-child string attributes through one local helper.
- Return static defaults for stepper, speed-dial, tab-scroll, and Drawer paths.
- Reuse borrowed Drawer values for component and slot class generation.

## Regression Contract

The shared `optimization_batch_20260826ea_` filter owns three Runtime tests: defaults/classes,
borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`RUNTIME170_SURFACE_CHILD_BORROWED_ATTRIBUTES_BENCH_V1`, performs 524,288 lookups per sample,
reduces lookup allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
