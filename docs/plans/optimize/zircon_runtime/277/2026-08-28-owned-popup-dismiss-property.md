# Runtime277 Owned Popup Dismiss Property

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime277-editor223-performance-batch-20260828ie-v1`

## Problem

Runtime accessibility popup dismissal received an owned reflected-property name from popup lookup
but cloned it when building the mutation request. The original property string was never used after
that call, so every dismissal duplicated its allocation before mutation.

## Optimization

- Move the lookup-owned property name directly into the accessibility mutation request.
- Keep RuntimeState source authority and AccessibilityAction binding-source classification.
- Preserve popup target selection, false-value mutation, report handling, and diagnostics.
- Isolate request construction so ownership and source semantics remain regression-testable.

## Regression Contract

The `optimization_batch_20260828ie_` Runtime tests prove mutation-request property allocation
identity and prevent the prior clone from returning. The ignored paired release benchmark emits
`RUNTIME277_OWNED_POPUP_DISMISS_PROPERTY_BENCH_V1`. It constructs 512 requests with 64-KiB property
names per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
