# Editor110 Component Variant Borrowed Attributes

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime164-editor110-performance-batch-20260826du-v1`

## Problem

Retained component variant projection cloned animation, color, orientation, alignment, and candidate
variant attributes before comparisons or appending them to the one final variant string. A
projection could allocate several temporary strings that never escaped the function.

## Optimization

- Borrow candidate variant text before creating the required final owned string.
- Borrow animation and role-specific attributes while appending tokens.
- Preserve invisible priority, animation de-duplication, and divider/timeline token behavior.

## Regression Contract

The shared `optimization_batch_20260826du_` filter owns three Editor tests: token behavior,
borrowed pointer/source shape, and an ignored paired release P50/P95 benchmark. The benchmark emits
`EDITOR110_COMPONENT_VARIANT_BORROWED_ATTRIBUTES_BENCH_V1`, performs 524,288 lookups per sample,
reduces lookup allocations from one to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
