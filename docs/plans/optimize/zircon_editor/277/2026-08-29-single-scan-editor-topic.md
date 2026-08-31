# Editor277 Single-Scan Editor Topic

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime332-editor277-performance-batch-20260829be-v1`

## Scope

Editor topic parsing previously searched for a separator, split the string, and then scanned each
segment again. Parsing now performs one byte-state scan that tracks separator presence, segment
boundaries, and the first validation error. Owned errors, error priority, accepted characters, and
the final owned topic remain unchanged.

## Static Evidence

- Validation passes over topic bytes: `up to 3 -> 1`.
- Temporary segment collections remain `0`.
- Invalid-segment allocation remains failure-only.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR277_SINGLE_SCAN_EDITOR_TOPIC_BENCH_V1`. It
compares the legacy contains/split/segment scans with the byte state scan over 8,192 valid 4-KiB
topics across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
