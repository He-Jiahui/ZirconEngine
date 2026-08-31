# Runtime271 Borrowed Submenu Current Option

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime271-editor217-performance-batch-20260828hy-v1`

## Problem

Opening a focused Runtime UI submenu already materialized option entries, then cloned every option ID
into a second vector solely to resolve the current index. The fallback lookup also copied the current
string value before comparing it with those cloned IDs.

## Optimization

- Resolve focused and selected indices directly before consulting string fallback values.
- Borrow fallback strings through the existing menu setting reference path.
- Scan the existing option-entry slice for the first matching ID without allocating another vector.
- Preserve index priority, first-match behavior, clamping, and submenu opening semantics.

## Regression Contract

The `optimization_batch_20260828hy_` Runtime tests prove first-match behavior and enforce the absence
of the cloned ID vector in the focused-submenu path. The ignored paired release benchmark emits
`RUNTIME271_BORROWED_SUBMENU_CURRENT_OPTION_BENCH_V1`. It resolves the last of 4,096 option IDs with
256-byte suffixes 64 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
