# Runtime278 Owned Expanded State Property

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime278-editor224-performance-batch-20260828if-v1`

## Problem

Runtime accessibility expand/collapse lookup materialized an owned reflected-property name and then
cloned it into the mutation request. The original property allocation was discarded immediately,
adding a full string copy to every disclosure and popup expanded-state action.

## Optimization

- Copy the small expandable action kind before consuming the lookup target.
- Move the owned property name directly into the accessibility mutation request.
- Preserve RuntimeState source authority and AccessibilityAction binding classification.
- Preserve action validation, component-event selection, mutation reporting, and diagnostics.

## Regression Contract

The `optimization_batch_20260828if_` Runtime tests prove mutation-request property allocation
identity and prevent the prior clone from returning. The ignored paired release benchmark emits
`RUNTIME278_OWNED_EXPANDED_STATE_PROPERTY_BENCH_V1`. It constructs 512 requests with 64-KiB property
names per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
