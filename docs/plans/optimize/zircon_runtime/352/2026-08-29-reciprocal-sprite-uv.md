# Runtime352 Reciprocal Sprite UV Projection

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime352-editor297-performance-batch-20260829by-v1`

## Scope

Sprite-atlas pixel-to-UV projection previously divided each of four coordinates independently.
Projection now computes one reciprocal per atlas axis and multiplies both minimum and maximum
coordinates by the shared reciprocal.

## Static Evidence

- Floating-point divisions per valid UV projection: `4 -> 2`.
- Coordinate multiplications per valid projection: `0 -> 4`.
- Overflow, zero extent, bounds, and coordinate result semantics remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME352_RECIPROCAL_SPRITE_UV_BENCH_V1`. It compares
four direct divisions with two reciprocals and four multiplications over 1,000,000 projections per
sample and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
