# Editor274 Common-Prefix Build Export Actions

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime329-editor274-performance-batch-20260829bb-v1`

## Scope

Build/export action parsing previously compared the complete `workbench.build_export.` prefix for
every action variant. Parsing now strips that common prefix once and matches only short variant
suffixes. Borrowed profile/output slices, variant priority, empty-value rejection, and unknown action
behavior remain unchanged.

## Static Evidence

- Common-prefix comparisons for the final `output.set` branch: `7 -> 1`.
- Parser allocations remain `0`.
- Returned profile and output-root slices still borrow the action identifier.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR274_COMMON_PREFIX_BUILD_EXPORT_ACTIONS_BENCH_V1`.
It compares repeated complete-prefix checks with one common-prefix strip over 131,072 final-branch
actions across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
