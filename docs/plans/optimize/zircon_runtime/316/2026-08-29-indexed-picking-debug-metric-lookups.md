# Runtime316 Indexed Picking Debug Metric Lookups

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime316-editor261-performance-batch-20260829ap-v1`

## Scope

`PickingDebugFeed::metric` previously scanned all six diagnostic metric rows for every lookup.
The feed constructor has a stable metric order, so the lookup now checks the expected slot first.
It validates the metric kind before returning and retains the original linear search as a fallback,
preserving behavior for callers that reorder or replace the public metric vector.

## Static Evidence

- Standard feed worst-case comparisons per lookup: `6 -> 1`.
- Additional retained indexes or allocations: `0`.
- Reordered public vectors continue through the legacy semantic fallback.

## Performance Gate

The ignored Windows release benchmark emits
`RUNTIME316_INDEXED_PICKING_DEBUG_METRIC_LOOKUPS_BENCH_V1`. It performs 250,000 terminal metric
lookups per sample across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
