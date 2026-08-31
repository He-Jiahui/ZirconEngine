# Editor208 Shared Source Buffer Text

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime262-editor208-performance-batch-20260826hp-v1`

## Problem

UI asset source buffers stored current and saved text as independent strings. Construction deep
copied the complete source into the saved slot, and every successful save repeated that full copy
even though both values represent the same immutable text revision.

## Optimization

- Store current and saved revisions as `Arc<String>` values.
- Share one allocation for clean buffers and use an Arc clone when marking a revision saved.
- Continue moving replacement strings into owned storage and preserve content-based dirty checks.

## Regression Contract

The `optimization_batch_20260826hp_` Editor tests preserve text, dirty-state, and revision behavior;
verify pointer sharing before edits and after save; enforce removal of both content-clone sites; and
provide an ignored paired release benchmark emitting `EDITOR208_SHARED_SOURCE_BUFFER_TEXT_BENCH_V1`.
It saves a 256 KiB source 128 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
