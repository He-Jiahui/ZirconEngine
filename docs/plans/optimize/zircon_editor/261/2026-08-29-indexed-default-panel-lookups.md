# Editor261 Indexed Default Panel Lookups

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime316-editor261-performance-batch-20260829ap-v1`

## Scope

The default Editor design stack contains 20 panels in a stable construction order, while
`EditorUiDesignStack::panel` previously scanned the full vector. Known default IDs now select their
expected slot directly, validate the stored ID, and fall back to the original scan for custom,
reordered, or replaced public panel vectors.

## Static Evidence

- Default terminal panel worst-case comparisons per lookup: `20 -> 1`.
- Additional retained indexes or allocations: `0`.
- Custom and reordered design stacks retain the original first-match behavior.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR261_INDEXED_DEFAULT_PANEL_LOOKUPS_BENCH_V1`.
It performs 100,000 terminal default-panel lookups per sample across 31 interleaved sample pairs and
requires `optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
