# Editor200 Preview Import Streaming Clone

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime254-editor200-performance-batch-20260826hh-v1`

## Problem

Preview compilation cloned each complete widget and style import `BTreeMap` before extending the
built-in import maps. That materialized an intermediate tree and then immediately consumed it,
duplicating node allocation and traversal for every preview rebuild.

## Optimization

- Stream borrowed import entries directly into each destination map.
- Clone only the key and document required by the destination entry.
- Preserve source-overrides-built-in ordering and leave the compiler import cache unchanged.

## Regression Contract

The `optimization_batch_20260826hh_` Editor tests preserve retained, new, and overriding import
semantics; enforce iterator-based entry cloning without whole-map clones; and provide an ignored
paired release benchmark emitting `EDITOR200_PREVIEW_IMPORT_STREAMING_CLONE_BENCH_V1`. It merges
512 imports repeatedly and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
