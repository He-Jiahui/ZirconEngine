# Editor258 Preallocated Native Window Targets

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime313-editor258-performance-batch-20260829am-v1`

## Problem

Native floating-window projection collected a filtered iterator into a zero-capacity vector even
though the workbench floating-window count was already the strict output upper bound. A frame with
many native windows repeatedly grew and copied the target list before host synchronization.

## Optimization

- Reserve the floating-window upper bound when the first valid native target is accepted.
- Preserve zero allocation when no floating window projects to a native host target.
- Preserve source order and every projection/native-host/surface-tree filter.
- Preserve cloned window identity, title, bounds, and surface tree ownership.

## Regression Contract

The `optimization_batch_20260829am_` Editor tests preserve filtered order and the empty-list fast
path, and guard the production reservation before the first append. The ignored paired release benchmark emits
`EDITOR258_PREALLOCATED_NATIVE_WINDOW_TARGETS_BENCH_V1`. It builds 10,000 filtered 512-window target
lists per sample, changes eight vector allocation operations to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
