# Runtime350 Cached Weight Axis Lookup

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime350-editor295-performance-batch-20260829bw-v1`

## Scope

Effective font variation projection previously scanned the face axis table to decide whether to
inject `wght`, then scanned the same table again to quantize that tag. The discovered weight axis is
now retained for the projection pass and reused when the canonical tag is processed.

## Static Evidence

- Injected weight-axis table traversals: `2 -> 1`.
- No auxiliary map or persistent allocation is introduced.
- Missing-axis, caller-authored weight, canonicalization, and default-value elision remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME350_CACHED_WEIGHT_AXIS_BENCH_V1`. It compares
the repeated lookup baseline with cached lookup over 512 axes, 4,096 projections per sample, and 31
interleaved sample pairs and requires `candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
