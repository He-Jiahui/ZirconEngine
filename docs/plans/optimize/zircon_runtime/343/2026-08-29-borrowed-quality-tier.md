# Runtime343 Borrowed Quality Tier Classification

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime343-editor288-performance-batch-20260829bp-v1`

## Scope

Shader prewarm quality-tier parsing previously allocated a lowercase copy for every CLI token. The
parser now trims once, dispatches by the small supported token lengths, and compares borrowed text
with ASCII case folding while preserving every tier expansion and usage error.

## Static Evidence

- Lowercase string allocations per classification: `1 -> 0`.
- Oversized invalid tokens are rejected after the length check without case-fold traversal.
- Low, medium, high, ultra, all, whitespace, case, and invalid behavior remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `RUNTIME343_BORROWED_QUALITY_TIER_BENCH_V1`. It compares
lowercase allocation with borrowed length-dispatched classification over 4,096-byte invalid tokens,
8,192 checks per sample, and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
