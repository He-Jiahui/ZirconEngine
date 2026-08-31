# Runtime281 Owned Stale Drag-Drop Event

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime281-editor227-performance-batch-20260828ii-v1`

## Problem

Runtime drag-drop dispatch cloned the complete input event before validating retained drag state.
A stale event therefore deep-copied its boxed payload and owned reference/source strings even
though the original event had no second consumer on that rejection path.

## Optimization

- Validate retained drag state before constructing the accepted-path event clone.
- Move stale drag-drop events and their boxed payload directly into the unhandled result.
- Preserve pointer routing, stale diagnostics, target selection, effects, and route annotation.

## Regression Contract

The `optimization_batch_20260828ii_` Runtime tests prove payload-reference allocation identity and
prevent the pre-check clone from returning. The ignored paired release benchmark emits
`RUNTIME281_OWNED_STALE_DRAG_DROP_EVENT_BENCH_V1`. It converts 512 events with 64-KiB payload
references per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
