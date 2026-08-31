# Runtime245 Font Coverage Hash Dedup

- Date: 2026-08-26
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime245-editor191-performance-batch-20260826gy-v1`

## Problem

Font coverage construction sorted every supplied codepoint before removing duplicates. Large
duplicate-heavy coverage inputs therefore paid sorting cost for repeated values even though range
compaction consumed only the much smaller unique codepoint stream.

## Optimization

- Preserve the existing in-place sort and dedup path below 128 codepoints.
- For larger inputs, insert codepoints into a pre-sized HashSet before materialization.
- Sort only unique codepoints before the existing contiguous-range compaction pass.

## Regression Contract

The `optimization_batch_20260826gy_` Runtime tests preserve canonical coverage ranges, enforce
hash-before-sort normalization, and provide an ignored paired release benchmark emitting
`RUNTIME245_FONT_COVERAGE_HASH_DEDUP_BENCH_V1`. It repeatedly normalizes 4,096 codepoints drawn
from 16 unique values and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns exact timings, record finalization,
manifest-only commit, push to `origin/main`, and one-shot WeCom after a pushed SHA exists.
