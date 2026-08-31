# Runtime267 Owned UI Binding Arguments

- Date: 2026-08-28
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime267-editor213-performance-batch-20260828hu-v1`

## Problem

UI binding invocation accepted an owned binding, cloned the complete argument vector for route
dispatch, and then discarded the original binding. Large string, array, record, or map arguments
were therefore deeply copied even though the invocation already owned their allocation.

## Optimization

- Move the argument vector out of a matched owned binding with `std::mem::take`.
- Preserve the complete original binding in the unknown-binding error path.
- Keep route lookup, default-route arguments, handler context, and notification delivery unchanged.

## Regression Contract

The `optimization_batch_20260828hu_` Runtime tests preserve argument values and prove that the
original `Vec` allocation is transferred without copying. They also enforce the owned invocation
source contract and provide an ignored paired release benchmark emitting
`RUNTIME267_OWNED_UI_BINDING_ARGUMENTS_BENCH_V1`. It hands off 512 arguments carrying 4 KiB strings
eight times per sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
