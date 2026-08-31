# Runtime233 Import Label Borrowed Set

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime233-editor179-performance-batch-20260826gm-v1`

## Problem

Asset import validation cloned every entry label into an owned HashSet even though the set lives
only for the duration of a function that already borrows the complete import outcome.

## Optimization

- Store borrowed label slices and reserve the entry-count upper bound before validation.
- Eliminate owned label clones from the successful unique-label path.
- Preserve duplicate detection order and retain an owned label in the returned duplicate error.

## Regression Contract

The `optimization_batch_20260826gm_` Runtime tests cover duplicate order and the borrowed-set source
contract, and provide an ignored paired release benchmark emitting
`RUNTIME233_IMPORT_LABEL_BORROWED_SET_BENCH_V1`. It validates 64 sets of 4,096 labels per sample
and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
