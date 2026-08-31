# Runtime280 Owned Stale Popup Event

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime280-editor226-performance-batch-20260828ih-v1`

## Problem

Runtime popup dispatch cloned the full input event before checking retained popup state. Stale close
and dismissal events therefore copied their owned popup ID even though the original event had no
second consumer on the rejection path.

## Optimization

- Check retained popup state before constructing the accepted-path event clone.
- Move stale popup events directly into the unhandled dispatch result.
- Preserve stale diagnostics, route-policy annotation, accepted popup effects, owner, and anchor.

## Regression Contract

The `optimization_batch_20260828ih_` Runtime tests prove stale-event popup-ID allocation identity
and prevent the pre-check clone from returning. The ignored paired release benchmark emits
`RUNTIME280_OWNED_STALE_POPUP_EVENT_BENCH_V1`. It converts 512 events with 64-KiB popup IDs per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
