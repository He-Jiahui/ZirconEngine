# Editor192 Active Tool Tab Single Pass

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime246-editor192-performance-batch-20260826gz-v1`

## Problem

Workbench active tool-tab resolution scanned the requested drawer slots once for the first visible,
non-collapsed stack and then scanned them again for the first visible fallback when every stack was
collapsed. Repeated fallback-only slot lists therefore doubled lookup and availability work.

## Optimization

- Traverse mapped tool-window stacks once.
- Remember the first visible nonempty collapsed stack as the fallback.
- Return immediately on the first visible nonempty non-collapsed stack, preserving strict priority.

## Regression Contract

The `optimization_batch_20260826gz_` Editor tests preserve strict-candidate and fallback ordering,
enforce the single-loop source shape, and provide an ignored paired release benchmark emitting
`EDITOR192_ACTIVE_TOOL_TAB_SINGLE_PASS_BENCH_V1`. It repeatedly selects from 4,096 fallback-only
stacks and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
