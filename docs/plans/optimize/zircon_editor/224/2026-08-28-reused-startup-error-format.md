# Editor224 Reused Startup Error Format

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime278-editor224-performance-batch-20260828if-v1`

## Problem

Editor startup-workbench failure handling formatted the same error twice: once for the welcome
session and again for the retained status line. It also replaced the startup status string instead
of reusing its existing buffer.

## Optimization

- Format the structured startup error exactly once.
- Update the welcome-session status with `String::clone_from` to reuse existing capacity.
- Move the single formatted string into status-line propagation.
- Preserve welcome refresh ordering, error text, retained status publication, and invalidation.

## Regression Contract

The `optimization_batch_20260828if_` Editor tests prove startup-status allocation identity and
prevent duplicate error formatting from returning. The ignored paired release benchmark emits
`EDITOR224_REUSED_STARTUP_ERROR_FORMAT_BENCH_V1`. It propagates 2,048 structured 128-segment errors
per sample, including the retained status copy, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
