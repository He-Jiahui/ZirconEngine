# Runtime334 Single-Scan Package Token Charset

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime334-editor279-performance-batch-20260829bg-v1`

## Scope

Runtime plugin package-token validation previously trimmed the token twice before invoking a
predicate that already rejects empty values, whitespace, and every non-token character. Validation
now invokes that predicate once. Accepted tokens, empty and whitespace rejection, Unicode handling,
and diagnostic text remain unchanged.

## Static Evidence

- `str::trim` calls per package token: `2 -> 0`.
- Full valid-token scans: `2 -> 1`.
- Invalid diagnostic allocation remains failure-only.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME334_SINGLE_SCAN_PACKAGE_TOKEN_BENCH_V1`. It
compares the baseline trim/equality/predicate path with the single predicate scan over 16,384 valid
4-KiB token checks across 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
