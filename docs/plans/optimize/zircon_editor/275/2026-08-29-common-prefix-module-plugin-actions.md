# Editor275 Common-Prefix Module Plugin Actions

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime330-editor275-performance-batch-20260829bc-v1`

## Scope

Module-plugin action parsing previously compared the complete `workbench.plugin.` prefix for every
action variant. Parsing now strips that common prefix once and matches short variant suffixes.
Feature dependency priority, borrowed plugin/feature slices, and unknown-action behavior remain
unchanged.

## Static Evidence

- Common-prefix comparisons for the final hot-reload branch: `9 -> 1`.
- Parser allocations remain `0`.
- Feature dependency parsing still precedes ordinary feature enable parsing.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR275_COMMON_PREFIX_MODULE_PLUGIN_ACTIONS_BENCH_V1`.
It compares repeated complete-prefix checks with one common-prefix strip over 131,072 final-branch
actions across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
