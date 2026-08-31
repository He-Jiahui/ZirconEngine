# Runtime311 Preallocated Solari Degradations

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime311-editor256-performance-batch-20260829ak-v1`

## Problem

Solari runtime reports appended up to six missing capability degradations and three provider or gate
degradations into an empty vector. A fully degraded report repeatedly grew and copied that vector.

## Optimization

- Derive the maximum degradation count from the six capability requirements plus three status
  conditions.
- Allocate the bounded report vector once before evaluating capability and provider state.
- Preserve degradation order, status precedence, and report contents.

## Regression Contract

The `optimization_batch_20260829ak_` Runtime tests verify the maximum-capacity calculation and the
production preallocation contract. The ignored paired release benchmark emits
`RUNTIME311_PREALLOCATED_SOLARI_DEGRADATIONS_BENCH_V1`. It builds 200,000 nine-item reports per
sample, changes three vector allocation operations to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
