# Editor297 Empty Inspector Value Short Circuit

- Implementation status: `implementation_complete`
- Managed validation: `managed_validation_pending`
- Validation request: `runtime352-editor297-performance-batch-20260829by-v1`

## Scope

Inspector row classification previously compared every resource and shadow label even when the
value was empty and those row kinds could not match. Empty values now take a dedicated Lighting-only
branch before non-empty resource and shadow classification.

## Static Evidence

- Empty unmatched label branches: `5 -> 1`.
- Lowercase allocations remain `0`.
- Lighting disclosure and every non-empty resource/shadow row mapping remain unchanged.

## Performance Gate

The ignored Windows release benchmark emits `EDITOR297_EMPTY_INSPECTOR_VALUE_SHORT_CIRCUIT_BENCH_V1`.
It compares the prior five label branches with the empty-value fast path over 1,000,000 checks per
sample and 31 interleaved sample pairs and requires
`candidate_p95_ns <= baseline_p95_ns * 0.70`.

No direct Cargo validation was started. The coordinator owns batched validation, exact timing
capture, record finalization, manifest-only staging, commit, push to `origin/main`, and the one-shot
WeCom result containing exact performance data, test result, commit SHA, and branch.
