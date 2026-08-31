# Editor203 Streaming V2 Style Merge

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime257-editor203-performance-batch-20260826hk-v1`

## Problem

V2 preview authoring cloned each imported style's complete token map and stylesheet vector before
extending the preview document. Those temporary container allocations were consumed and released
immediately for every imported style on every preview compilation.

## Optimization

- Stream borrowed token entries directly into the preview document map.
- Clone stylesheet entries directly from their borrowed slice into the destination vector.
- Preserve BTreeMap import order, later-token override semantics, and stylesheet ordering.

## Regression Contract

The `optimization_batch_20260826hk_` Editor tests preserve base/style-a/style-b token override and
stylesheet order; enforce iterator-based map and slice cloning without whole-container clones; and
provide an ignored paired release benchmark emitting `EDITOR203_STREAMING_V2_STYLE_MERGE_BENCH_V1`.
It merges 512 token entries 64 times per sample and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
