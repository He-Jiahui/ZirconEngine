# Runtime295 Allocation-Free Style-Token Path Match

- Date: 2026-08-29
- Session: `root-runtime-editor-optimize-20260828-r3`
- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime295-editor241-performance-batch-20260829v-v1`

## Problem

Removing stale style-token sources allocated two temporary strings on every call: one for the
dotted descendant prefix and one for the indexed descendant prefix. The map scan only needed to
distinguish the first borrowed character after an existing path prefix.

## Optimization

- Compare the exact path before examining descendants.
- Strip the candidate prefix as a borrowed slice.
- Treat only `.` and `[` as descendant boundaries.
- Preserve similarly prefixed sibling keys without allocating prefix strings.

## Regression Contract

The `optimization_batch_20260829v_` Runtime tests cover exact, dotted, indexed, sibling, and
unrelated paths and guard the borrowed-suffix implementation. The ignored paired release benchmark
emits `RUNTIME295_ALLOCATION_FREE_STYLE_TOKEN_PATH_MATCH_BENCH_V1`. It performs 100,000 match
batches across eight candidates and a 53-byte target per sample, reduces prefix allocations per
batch from two to zero, and requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

## Validation Ownership

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only commit, push to `origin/main`, and one-shot WeCom after
a pushed SHA exists.
