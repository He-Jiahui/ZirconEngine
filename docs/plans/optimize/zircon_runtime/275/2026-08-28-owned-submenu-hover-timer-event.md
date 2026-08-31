# Runtime275 Owned Submenu Hover Timer Event

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime275-editor221-performance-batch-20260828ic-v1`

## Problem

Runtime submenu-hover timer dispatch received an owned input DTO but cloned it before retained-state
checks and the component default action. That duplicated the option-ID allocation even though those
operations only borrowed the DTO and the original value was discarded after dispatch.

## Optimization

- Borrow the option ID and target for retained-state and default-action processing.
- Copy the node target and move the complete DTO into the returned input event afterward.
- Move directly into the stale result when retained submenu-hover state no longer matches.
- Preserve stale diagnostics, handled routing, component events, and route-policy annotation.

## Regression Contract

The `optimization_batch_20260828ic_` Runtime tests prove that the returned event retains the owned
option-ID allocation and prevent the prior DTO clone from returning. The ignored paired release
benchmark emits `RUNTIME275_OWNED_SUBMENU_HOVER_TIMER_EVENT_BENCH_V1`. It converts 512 timer inputs
with 64-KiB option IDs per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
