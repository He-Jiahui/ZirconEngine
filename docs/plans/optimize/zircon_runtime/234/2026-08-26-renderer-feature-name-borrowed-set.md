# Runtime234 Renderer Feature Name Borrowed Set

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260826-r2`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime234-editor180-performance-batch-20260826gn-v1`

## Problem

Renderer data document validation cloned every feature name into an owned BTreeSet even though the
set is local to a function that already borrows the complete feature slice.

## Optimization

- Store borrowed feature-name slices in the uniqueness set.
- Eliminate one owned String clone per feature from the successful validation path.
- Preserve duplicate order and clone only the duplicate name required by the owned error value.

## Regression Contract

The `optimization_batch_20260826gn_` Runtime tests exercise the real duplicate error and the
borrowed-set source contract, and provide an ignored paired release benchmark emitting
`RUNTIME234_RENDERER_FEATURE_NAME_BORROWED_SET_BENCH_V1`. It validates 64 sets of 4,096 names per
sample and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
