# Editor278 Single-Scan Operation Path

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime333-editor278-performance-batch-20260829bf-v1`

## Scope

Editor operation-path validation previously split the path and decoded every segment as characters.
Validation now performs one byte-state scan that counts segments, rejects empty boundaries, and
validates the ASCII operation alphabet. Minimum segment count, accepted characters, invalid-path
errors, and owned parse results remain unchanged.

## Static Evidence

- Validation passes over operation-path bytes: `up to 2 -> 1`.
- Temporary segment collections remain `0`.
- Invalid-path allocation remains failure-only.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR278_SINGLE_SCAN_OPERATION_PATH_BENCH_V1`. It
compares the baseline split/character-decode path with the byte-state scan over 8,192 valid 4-KiB
operation paths across 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
