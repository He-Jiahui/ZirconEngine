# Runtime274 Owned Toast Timer Event

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime274-editor220-performance-batch-20260828ib-v1`

## Problem

Runtime toast-timer dispatch received an owned input DTO but cloned the complete DTO before running
the component default action. That duplicated the toast-ID allocation even though dispatch only
needed to borrow the original fields first and discarded the original DTO afterward.

## Optimization

- Run the component default action while borrowing the owned toast input.
- Copy the node target and then move the complete DTO into the returned input event.
- Preserve stale-timer diagnostics, handled routing, component events, and route-policy annotation.
- Remove one duplicate toast-ID allocation from both successful and stale timer dispatch paths.

## Regression Contract

The `optimization_batch_20260828ib_` Runtime tests prove that the returned event retains the owned
toast-ID allocation and prevent the prior DTO clone from returning. The ignored paired release
benchmark emits `RUNTIME274_OWNED_TOAST_TIMER_EVENT_BENCH_V1`. It converts 512 timer inputs with
64-KiB toast IDs per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
