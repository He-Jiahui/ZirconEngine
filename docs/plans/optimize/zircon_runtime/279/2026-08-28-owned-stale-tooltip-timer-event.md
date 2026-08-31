# Runtime279 Owned Stale Tooltip Timer Event

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime279-editor225-performance-batch-20260828ig-v1`

## Problem

Runtime tooltip-timer dispatch cloned the complete input event before checking retained tooltip
state. A stale elapsed or canceled timer therefore copied its owned tooltip ID even though the
original event had no second consumer on that rejection path.

## Optimization

- Check retained tooltip state before constructing the accepted-path event clone.
- Move stale timer events directly into the unhandled dispatch result.
- Preserve stale diagnostics, route-policy annotation, accepted tooltip effects, and ownership.

## Regression Contract

The `optimization_batch_20260828ig_` Runtime tests prove stale-event tooltip-ID allocation identity
and prevent the pre-check clone from returning. The ignored paired release benchmark emits
`RUNTIME279_OWNED_STALE_TOOLTIP_TIMER_EVENT_BENCH_V1`. It converts 512 events with 64-KiB tooltip
IDs per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
