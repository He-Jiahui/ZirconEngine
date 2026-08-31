# Editor283 Single-Scan Chip Slot

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime338-editor283-performance-batch-20260829bk-v1`

## Scope

Retained chip-slot classification previously tokenized the same component-variant string three
times for case-insensitive exact matches and a fourth time for the `chipSlot` prefix. Classification
now resolves the canonical exact token and prefix family in one token traversal. Delimiters,
case-insensitivity, exact MUI slot matching, and derived chip-slot prefixes remain unchanged.

## Static Evidence

- Component-variant token traversals per unmatched chip slot: `4 -> 1`.
- Duplicate case-insensitive exact conditions: `2 -> 0`.
- Temporary token collections remain `0`.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR283_SINGLE_SCAN_CHIP_SLOT_BENCH_V1`. It compares
the baseline four token traversals with the combined traversal over 8,192 unmatched 4-KiB variants
across 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
