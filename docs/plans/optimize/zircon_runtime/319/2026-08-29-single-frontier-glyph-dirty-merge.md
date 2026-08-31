# Runtime319 Single-Frontier Glyph Dirty Merge

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime319-editor264-performance-batch-20260829as-v1`

## Scope

Non-replayable glyph atlas pages previously rescanned every dirty-region pair after each new
region, although the previous region set was already at a merge fixed point. Dirty tracking now
searches only pairs involving the new or newly merged frontier. Replayable-shadow pages retain the
full scan because write-limit compaction can introduce a frontier outside the current call.

## Static Evidence

- Pair checks for 160 non-merging regions: `682640 -> 12720` per page build.
- Theoretical pair-check reduction: `98.14%`.
- Merge selection, retained-pixel protection, and replayable-shadow behavior are unchanged.

## Performance Gate

The ignored Windows release benchmark emits
`RUNTIME319_SINGLE_FRONTIER_GLYPH_DIRTY_MERGE_BENCH_V1`. It builds four 160-region pages per sample
across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
