# Editor252 Single-Allocation Material Lab Bindings

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime306-editor252-performance-batch-20260829ag-v1`

## Problem

Material Lab binding construction collected the primary specifications into an exactly sized
vector, then extended it with structural-child bindings. The second group forced the completed
primary buffer to grow and move its entries.

## Optimization

- Reserve the combined primary and structural specification count once.
- Extend both mapped specification groups into that final buffer.
- Preserve every binding value and the original group ordering.

## Regression Contract

The `optimization_batch_20260829ag_` Editor tests compare the complete optimized and legacy binding
vectors and guard the single-allocation source contract. The ignored paired release benchmark
emits `EDITOR252_SINGLE_ALLOCATION_MATERIAL_LAB_BINDINGS_BENCH_V1`. It uses the production group
sizes for 100,000 builds per sample, reduces buffer growth operations from two to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
