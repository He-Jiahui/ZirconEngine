# Runtime339 Single-Append Zcube Encoding

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime339-editor284-performance-batch-20260829bl-v1`

## Scope

`.zcube` texture encoding previously allocated an encoded texel buffer and then copied it into a
second container buffer. Encoding now reserves the final payload capacity once and appends RGBA16F
texels directly. Header bytes, payload bytes, texture metadata, and decode behavior remain unchanged.

## Static Evidence

- Payload allocations per source cubemap encode: `2 -> 1`.
- Full payload copies: `1 -> 0`.
- Existing RGBA16F conversion routine remains the single encoder.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME339_SINGLE_APPEND_ZCUBE_BENCH_V1`. It compares
the baseline temporary-encode/copy path with direct append over 8,192 texels for 64 checks across 31
interleaved sample pairs and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
