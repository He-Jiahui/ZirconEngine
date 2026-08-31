# Editor282 Single-Scan Badge Slot

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime337-editor282-performance-batch-20260829bj-v1`

## Scope

Retained badge-slot classification previously tokenized the same component-variant string three
times, including two case-insensitive conditions that represented the same token. Classification
now recognizes the MUI and regular badge slot forms in one token traversal. Delimiters,
case-insensitivity, exact matching, and non-slot rejection remain unchanged.

## Static Evidence

- Component-variant token traversals per unmatched badge slot: `3 -> 1`.
- Duplicate case-insensitive exact conditions: `2 -> 0`.
- Temporary token collections remain `0`.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR282_SINGLE_SCAN_BADGE_SLOT_BENCH_V1`. It compares
the baseline three token traversals with the combined traversal over 8,192 unmatched 4-KiB variants
across 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
