# Runtime345 Borrowed Codepoint Range Endpoints

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime345-editor290-performance-batch-20260829br-v1`

## Scope

Font SDF codepoint-range parsing previously copied both split endpoint slices into `OsString` and
then back into `String` before hexadecimal parsing. Single-codepoint and range paths now share a
borrowed `&str` parser while preserving validation and diagnostic text.

## Static Evidence

- Endpoint ownership conversions per range: `2 -> 0`.
- Output vector allocation and scalar validation remain unchanged.
- Upper/lower prefix, reversed range, invalid scalar, and valid range behavior remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME345_BORROWED_CODEPOINT_RANGE_BENCH_V1`. It
compares owned endpoint round-trips with borrowed endpoint parsing over a one-scalar range, 8,192
checks per sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
