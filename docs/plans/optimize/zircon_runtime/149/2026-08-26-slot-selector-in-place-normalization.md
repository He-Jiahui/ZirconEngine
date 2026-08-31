# Runtime149 Slot Selector In-Place Normalization

- Date: 2026-08-26
- Session: `root-runtime-events-20260824`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime149-editor95-performance-batch-20260826df-v1`

## Problem

Runtime session slot and tag selector constructors converted inputs into owned strings, then called
`trim().to_string()` and discarded the first allocation. Owned inputs therefore paid a second
allocation before every archive query.

## Optimization

- Convert each input once, then truncate and drain whitespace in the owned buffer.
- Share one normalizer across slot ID, latest-tag, and oldest-tag selector constructors.
- Preserve selector variants, Unicode trimming, and borrowed-input behavior.

## Regression Contract

The shared `optimization_batch_20260826df_` filter owns three Runtime tests: constructor behavior,
owned-buffer reuse plus shared-source shape, and an ignored paired release P50/P95 benchmark. The
benchmark emits `RUNTIME149_SLOT_SELECTOR_IN_PLACE_NORMALIZATION_BENCH_V1`, constructs 16,384
selectors per sample, records secondary trim allocations from 16,384 to zero, and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The validation coordinator owns the immutable Windows
release run, exact P50/P95 and reduction backfill, manifest-only stage, commit, push to
`origin/main`, and the one-shot WeCom report after a pushed SHA exists.
