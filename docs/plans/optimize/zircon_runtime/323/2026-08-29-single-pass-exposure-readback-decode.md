# Runtime323 Single-Pass Exposure Readback Decode

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime323-editor268-performance-batch-20260829aw-v1`

## Scope

Exposure readback decoding previously populated the four-word buffer and then traversed it again to
count non-finite values. Decoding now increments the invalid-word count while each available word is
materialized. Exact byte-length reporting, short-read zero fill, bit projections, and history
validity behavior are unchanged.

## Static Evidence

- Word traversals per exposure report: `2 -> 1`.
- Added heap allocations per report: `0`.
- Missing words remain finite zero values and do not increase the invalid-word count.

## Performance Gate

The ignored Windows release benchmark emits
`RUNTIME323_SINGLE_PASS_EXPOSURE_READBACK_DECODE_BENCH_V1`. It decodes four words 262,144 times per
sample across 31 interleaved sample pairs and requires
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
