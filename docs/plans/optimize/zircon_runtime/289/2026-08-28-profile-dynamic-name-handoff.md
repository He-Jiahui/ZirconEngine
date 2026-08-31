# Runtime289 Profile Dynamic-Name Handoff

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime289-editor235-performance-batch-20260828iq-v1`

## Problem

When Tracy and the built-in profiler were enabled together, every dynamic scope first owned one
String for tracing and then cloned that String for the built-in scope. Render graph stage and pass
instrumentation therefore performed two name allocations per active dynamic span.

## Optimization

- Create the Tracy span while it can borrow the dynamic name.
- Move the same owned String into the built-in profile scope after tracing has captured its fields.
- Preserve both scope guards and the inactive-capture behavior under existing feature gates.

## Regression Contract

The `optimization_batch_20260828iq_` Runtime tests guard the borrow-before-move macro order and both
sink-visible lengths. The ignored paired release benchmark emits
`RUNTIME289_PROFILE_DYNAMIC_NAME_HANDOFF_BENCH_V1`. It performs 100,000 handoffs of a 672-byte name
per sample, reduces name allocations from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
