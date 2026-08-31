# Editor253 Single-Buffer Export Preset Staging Name

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260829-r5`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime307-editor253-performance-batch-20260829ah-v1`

## Problem

Export preset transactions formatted process and thread identity into a nonce string, copied the
destination file name into another string, then formatted both into the final staging name. The
constructor also copied the parent path before immediately joining the staging name.

## Optimization

- Borrow the destination file name and parent path for the duration of construction.
- Format process id, nonce, and file name directly into the final staging-name string.
- Build the staging path before moving the destination into the transaction.

## Regression Contract

The `optimization_batch_20260829ah_` Editor tests cover exact staging names and guard the
single-buffer constructor contract. The ignored paired release benchmark emits
`EDITOR253_SINGLE_BUFFER_EXPORT_PRESET_STAGING_NAME_BENCH_V1`. It builds 150,000 staging names per
sample, reduces staging-name string allocations from three to one, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
