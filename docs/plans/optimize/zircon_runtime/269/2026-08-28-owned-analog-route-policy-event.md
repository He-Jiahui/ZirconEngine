# Runtime269 Owned Analog Route Policy Event

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime269-editor215-performance-batch-20260828hw-v1`

## Problem

Analog input dispatch already owned its final `UiInputEvent`, but route-policy annotation cloned the
complete event to work around borrowing the event and dispatch result at the same time. Metadata
and control strings were therefore reallocated on every analog route before the copy was dropped.

## Optimization

- Move the owned event out of the dispatch result through a non-allocating analog placeholder.
- Annotate route policy against the moved event and restore it before route-step annotation.
- Preserve navigation conversion, route policy, trace construction, diagnostics, effects, and the
  exact final event value.

## Regression Contract

The `optimization_batch_20260828hw_` Runtime tests prove allocation identity for moved control and
window-id strings and enforce the move-and-restore source contract. The ignored paired release
benchmark emits `RUNTIME269_OWNED_ANALOG_ROUTE_POLICY_EVENT_BENCH_V1`. It processes an analog event
with three 64 KiB string allocations 128 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
