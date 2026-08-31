# Runtime351 Segmented Cluster Extend

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime351-editor296-performance-batch-20260829bx-v1`

## Scope

Virtual-geometry cluster collection previously flattened matching instance slices into an iterator
whose aggregate size was unknown to Vec collection. The populated-instance path now extends the
output one exact slice at a time before retaining the existing final sort and deduplication.

## Static Evidence

- Per-instance cluster append size is exact at each `extend` call.
- Invalid cluster ranges remain skipped.
- Stable-key filtering, cluster order before sort, final sorting, and deduplication remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME351_SEGMENTED_CLUSTER_EXTEND_BENCH_V1`. It
compares flat-map collection with segmented exact-size extension over 256 instances with 32 clusters
each, 1,024 collections per sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
